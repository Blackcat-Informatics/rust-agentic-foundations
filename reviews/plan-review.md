# Plan review: Shared crates `raf-okf` + `raf-source`

**Reviewed plan:** `/home/paudley/Active/rust-agentic-foundations/.worktrees/2-shared-crates-raf-okf-raf-source/.stage/shared-crates-raf-okf-raf-source/plan.md`
**Reviewed:** 2026-10-12
**Base:** `origin/main` 2c41082, branch `paudley/2-shared-crates-raf-okf-raf-source`

## Coverage

Read in full: `plan.md`, `issue.md`, `issue-analysis.md` (I-1..I-24, O-1..O-24, S-1..S-34,
A-1..A-9, C-1..C-18, R-1..R-15), `prior-art-assessment.md` (L1-L14, port-vs-build table,
P1-P16), `AGENTS.md`, `CLAUDE.md`, `docs/capabilities/okf-profile.md`,
`docs/capabilities/source-index.md`, `docs/architecture/okf-profile.md`,
`work/changes/okf-profile.md`, `work/changes/source-index.md`, `Makefile`, `Cargo.toml`,
`deny.toml`, `.github/workflows/ci.yaml`, `.githooks/pre-commit`, `REUSE.toml`,
`rust-toolchain.toml`, `tests/support/mod.rs`. Read in part, at the lines the analysis
cites plus the call sites that decide scan scope: `tests/docs_contract.rs` (header, INVENTORY,
`parse_map`, `parse_inline_list`, `is_scanned`, `check_binding_contents`,
`check_capability_state`, `check_inventory`, `check_links` and its call site at 1687-1688,
`inventory_table_has_the_documented_shape`), `tests/repo_gates.rs` (issue-refs gate and its
fixtures), `SECURITY.md` (trust model 37-66, surface 77-84),
`docs/capabilities/workplan-document-model.md` (binding resolution 98-104),
`docs/capabilities/census-ledger.md` (43-45, 160-166). Confirmed: no `crates/`, no `fuzz/`,
no `docs/capabilities/index.md`, no `.baseline`, `.goals`, `CONSTITUTION.md` or ADRs.

Every issue requirement (I-1..I-24) and every Contract, Constraints and Verification
statement of both capability documents (O-*, S-*, A-*) was walked against the plan's
completeness table and task scopes. Four statements have no falsifiable check or no task
(findings F-1..F-4). Everything else maps to a task and a check on a shipped entry point.
No AGENTS.md rule violation was found; the one scoped `#[allow]` sits inside the rule that
permits it (F-7).

## Findings

### F-1 (required) UNMAPPED REQUIREMENT: frontmatter-less Markdown has no body-model entry point after Task 8

`tests/docs_contract.rs:1687-1688` runs `check_links` over **every** tracked Markdown file
(`README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `.github/*.md`, `docs/*.md` translations), and
1690-1693 runs `scan_markers` over every top-level, `docs/` and `work/` Markdown file. None of
those carry frontmatter, and `Document::parse` "fails ... when the file has no frontmatter
fence on line 1" (okf-profile.md:33-35). `Index::parse` is specific to the capability index.
So after Task 8 the link and marker scan of plain Markdown has nowhere to go: either
`docs_contract.rs` keeps a private scanner, which makes "The body model exists so that no
other crate scans Markdown" (okf-profile.md:137) and "`raf-okf` is the one reader ... the
repository tooling alike, goes through it" (okf-profile.md:20-23) false at `delivered`, or an
API appears unplanned in Task 8.

Offending plan lines (320-326):

> `tests/docs_contract.rs` replaces its frontmatter model, YAML subset parser, fence tracking,
> section and marker scanning, link extraction, per-document checks and index rule with calls
> to `raf_okf` (parse, body model, `Profile::validate`, `Index::parse`), keeping only
> cross-document checks, link resolution against tracked files, inventory, placeholder and
> banned-wording scans.

`REWRITE TO:` Task 2 scope adds `raf_okf::Body::parse(text) -> Result<Body, ParseError>`, the
body model over Markdown with no frontmatter (fences, sections, links of every C-18 form,
generated and PROPOSED blocks, the 64-level block bound); `Document::parse` builds its body
through it. Task 8 scans every frontmatter-less tracked Markdown file for links and markers
through `Body::parse`; `docs_contract.rs` keeps no link, fence or marker scanner of its own.
Fixtures: README-shaped text with `[a](b.md)` inside a fence and one outside yields exactly
the outside link; a PROPOSED marker in such a file is still a block (so `docs_contract` can
keep refusing markers outside capabilities). `okf-profile.md` Contract "Body model" and Usage
name `Body::parse`.

### F-2 (required) UNMAPPED REQUIREMENT: the Examples block uses a flow mapping the subset refuses

`okf-profile.md:110` is `bindings: {code: [], tests: [], examples: []}`. The profile's own
scalar rule says a plain value "contains no `: `" (architecture 76-78), and the existing
checker's `parse_map` (docs_contract.rs:313-327) reads block form only, so this line is
refused by the profile as written today; it survives only because it sits in a fence. Task 1
lists "block mappings, block and flow sequences of scalars, ... a nested map for `bindings`"
and never admits flow mappings (prior-art P11 says refuse them), yet the completeness row
asserts the block validates clean. Both cannot hold. If flow mappings were admitted instead,
the writer would have to pick one canonical form and the Example would no longer be
canonical for the round-trip law.

Offending plan line (158, Acceptance check column):

> the Examples YAML validates clean and with `resource` names the key set

`REWRITE TO:` Task 1: a flow mapping (`{`) as a value is outside the subset and is refused
with a positioned error naming the key; fixture `bindings: {code: []}` fails at its line and
the neighbouring block-form fixture parses. Task 3 docs: the Examples block of
`okf-profile.md` is rewritten to block form (`bindings:` / `  code: []` / `  tests: []` /
`  examples: []`), and the Examples test validates that exact text clean and, with a
`resource:` line added, names `resource` and the seven Capability keys.

### F-3 (required) UNMAPPED REQUIREMENT: the `toml` recursion limit has no fixture

source-index.md:70-72: "`Cargo.toml` is read with the same size cap, and the `toml` parser's
own recursion limit turns deep nesting into an error and not a crash." The size-cap half is
mapped; the recursion half is not. S-13 in the analysis names the check (a deeply nested
inline-table manifest). The completeness row cites S-13 but its check column covers only
bytes.

Offending plan line (163):

> | `max_file_bytes` default 8 MiB, `LoadOptions`, error names file; Cargo.toml same cap (S-8, S-13) | 4 (cap in `read`), 6 (manifest) | 8 MiB+1 fails naming the file, lowered cap honoured, 8 MiB parses |

`REWRITE TO:` add to the S-13 check: Task 6 fixture of a `Cargo.toml` with inline tables
nested past the `toml` parser's limit (for example 1000 levels, under the size cap) makes
`Workspace::load` return a typed manifest error naming the file, and a neighbouring 8-level
manifest loads; the test runs on the caller's 2 MiB stack so a crash cannot be mistaken for
the error.

### F-4 (required) UNMAPPED REQUIREMENT: "the documentation of the capabilities that rely on the index says so"

source-index.md:103-104: "A symbol that a macro generates is not indexed, and the
documentation of the capabilities that rely on the index says so." `census-ledger.md:164-165`
already says it for homes. `workplan-document-model.md:98-104` (binding resolution through the
source index) does not. The plan's row for S-22 maps the first clause only, and Task 9 edits
AGENTS.md, README.md, the architecture page, SECURITY.md and CHANGELOG.md, not the
document-model capability. At `delivered` the Contract sentence would be false.

Offending plan line (172, requirement column):

> Use-alias table: ... macro-generated and dependency items not found (I-18, I-21, S-21, S-22, S-24, C-14)

`REWRITE TO:` Task 9 adds to `docs/capabilities/workplan-document-model.md` Contract "Binding
resolution" the sentence "A symbol that a macro generates is not in the index, so it cannot
be bound." (census-ledger already states the rule for homes), and the S-22 row cites both
documents. Check: `cargo test --test docs_contract` stays green and the new docs-contract
binding test refuses a binding to a `macro_rules!`-generated function with `NotFound`.

### F-5 (advisory) `parse(render(d)) == d` and key order are not jointly specified

Plan line 154 asserts, for out-of-order input, "renders to profile order and re-renders
identically; `parse(render(d)) == d`". If `Document` keeps input key order (it must, to render
unknown-type documents in input order) and `PartialEq` is derived, the second property fails
for exactly the out-of-order fixtures. State the equality: frontmatter compared as a key to
value map plus body bytes, order ignored; or normalise known types to profile order at parse.
Either is fine; pick one in Task 1 and say it in the Contract "Write" paragraph.

### F-6 (advisory) name the Contract sentence that the residual race replaces

source-index.md:46-47 reads "No check separates from the open it guards." The plan's design
(canonicalise, prefix-check, open `O_NOFOLLOW`, `fstat`) keeps a containment check that is
separate from the open; the sentence is true only of the file-type check. Task 4 docs list
"the residual race" under Contract "Path containment", which is the right place, but the plan
should say that this sentence is replaced, not merely supplemented, so a reader of the
delivered document cannot find a stronger claim than the build makes. Suggested text: "The
file type is checked on the opened handle, never by a stat before the open. Containment is
checked on the canonical path before the open; a rename of an intermediate directory between
those two steps by a writer with access to the tree is a residual race, outside the trust
model in SECURITY.md."

### F-7 (advisory) the counting allocator: within the rule, but a dependency removes the only `unsafe`

AGENTS.md:124-125: "`allow_attributes_without_reason` is denied: every `#[allow]` states why,
scoped as tightly as possible. Prefer fixing the code." A scoped
`#[allow(unsafe_code, reason = "...")]` on one `GlobalAlloc` impl in one raf-okf test binary
does not edit `[workspace.lints]` and is what that rule permits; CLAUDE.md's "never weaken a
workspace lint" is not breached. "Prefer fixing the code" has a cheap form here: a
dev-dependency that owns the `unsafe` (for example `cap`, MIT OR Apache-2.0, no dependencies,
a peak counter) leaves the workspace with zero `unsafe` tokens and zero `unsafe_code` allows.
Cost: one dev-dependency in the deny graph. Take it if the crate's peak counter is confirmed;
otherwise keep the plan's scoped allow and state in the reason why no safe path exists.

### F-8 (advisory) the excluded fuzz workspace is outside every gate but its own run

With `exclude = ["fuzz"]`, `cargo fmt --all`, clippy, `make msrv` and `cargo deny` never see
`fuzz/`; `[lints] workspace = true` cannot be inherited there. The target is a few lines, but
the gap should be closed where it is cheap: the `fuzz` CI job and Makefile target also run
`cargo fmt --check` and `cargo clippy -- -D warnings` with `--manifest-path fuzz/Cargo.toml`,
and `cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml check` with a crate-scoped
NCSA exception for `libfuzzer-sys`, so the fuzz graph is judged rather than invisible. Pin the
`cargo-fuzz` version in CI (`cargo install --locked cargo-fuzz@<ver>`) as `cargo-deny@0.20.2`
is pinned. Pass a fixed `-seed` in `make fuzz` so a gate in `make check` is reproducible, and
keep unseeded exploration a separate invocation. Update the gate list at AGENTS.md:97 and the
Makefile header comment to name `test-release` and `fuzz`.

### F-9 (advisory) lockfiles and the `--locked` hook

Every gate and the hook run `--locked`. Task 1 (new member), Task 4 (`libc`), Task 5 (`syn`,
`proc-macro2`, fuzz workspace), Task 6 (`toml`) and Task 8 (root dev-dependencies) each change
the graph; each of those commits must stage the root `Cargo.lock` (and Task 5 `fuzz/Cargo.lock`)
or the next `cargo test --locked` fails on a clean checkout even though the hook, which
compiles from the working tree, stayed green. Say "stage `Cargo.lock`" in those task scopes.
Also note the cost Task 8 adds to every commit: the hook now compiles `raf-source`, `syn` and
`toml` into `docs_contract`; cold once, incremental after.

### F-10 (advisory) `replace_span` after a replacement

Plan line 206: "`Document::replace_span` splices a caller-supplied byte range of the body."
After one splice every later span in the body model is stale. Say which it is: the model is
rebuilt after each splice (spans re-read from the document), or callers apply replacements
from the last span to the first. The generator that writes several generated blocks in one
document will hit this.

### F-11 (advisory) the root package becomes unix-only

`compile_error!` on non-unix in `raf-source` plus the Task 8 dev-dependency means the root
package's tests no longer build on Windows. CI is ubuntu only and nothing in AGENTS.md
promises more, so this is acceptable, but Task 9 should state it where the toolchain is
described (AGENTS.md section 2 or SECURITY.md), not only in the source-index Constraints.

### F-12 (advisory) the two policy decisions need their governing lines in the handoff

The `unicode-ident` exception (deny.toml:10-11 asks for "someone to look at the new crate and
decide") and the base-OKF key decision (issue:19 against architecture 22-28 and
okf-profile.md:123) are both taken by the plan and surfaced "in the PR". The PR text should
quote those lines and the licence expression `(MIT OR Apache-2.0) AND Unicode-3.0`, so the
maintainer's merge is the deliberate look the comment asks for.

## Judgements requested

**(a) Targeting `delivered`.** Correct, and the stronger of the two states the issue allows.
Grounds: AGENTS.md:199-201 "No deferrals" and :183-186 (completion deletes the record, every
item has an outcome) make `implementing` legitimate only if part of the scope moved to another
tracked change; nothing here needs moving. The plan maps every Verification bullet, including
the release run, the child-process harness and the fuzz target, to a gate inside `make check`,
which is the test `delivered` must pass (issue-analysis R-13). Consequences the plan accepts
and states: `make check` now needs `cargo-fuzz` (consistent with AGENTS.md:112 "A missing tool
is a hard failure"), the inventory shape test changes its counts, two records are deleted,
and AGENTS.md/README/architecture prose is rewritten in the present tense. The decision stands
once F-1..F-4 are closed; those are the gaps between the mapped table and the documents.

**(b) The crate-scoped `unicode-ident` exception.** Acceptable and correctly shaped. `syn` is
mandated by the Contract ("parses each file with `syn`", source-index.md:83), `proc-macro2`
always pulls `unicode-ident`, and Unicode-3.0 is a permissive data-table licence. A
`[[licenses.exceptions]] crate = "unicode-ident" allow = ["Unicode-3.0"]` entry is evaluated per
licence requirement, so `(MIT OR Apache-2.0) AND Unicode-3.0` passes with the global `MIT`
and the scoped `Unicode-3.0`, and no other crate gains the licence. That is the "deliberate
look" deny.toml:10-11 asks for, provided the comment above the entry gives the reason in
system terms (no process wording, per `tests/support/mod.rs:30-87`) and F-12 is done. Not a
gate weakening under AGENTS.md:110-111: the gate still judges every crate.

**(c) The scoped `#[allow(unsafe_code, reason)]`.** Within AGENTS.md:124-125; see F-7 for the
no-`unsafe` alternative. One `#[global_allocator]` per test binary is required, so "its own
test file" is correct.

**(d) Fuzz exclusion and the new gates.** The exclusion is the right shape (cargo-fuzz wants
its own workspace; nightly-only and NCSA stay out of `make msrv` and the root deny graph). The
new `test-release` and `fuzz` gates strengthen `make check` and match source-index.md:181-182,
197, 209-210. Gaps: F-8 (fuzz crate outside fmt/clippy/deny, unpinned cargo-fuzz, unseeded
run inside a gate, stale gate list in AGENTS.md:97).

**(e) Task 8.** Required, not scope creep. okf-profile.md:20-23 makes raf-okf "the one reader
and writer of RAF documents ... the repository tooling alike"; at `delivered` that sentence
must describe the build, and `docs_contract.rs` is the repository tooling. Keeping two
interpreters would also be the duplication AGENTS.md:34-35 says the census exists to catch.
The plan's constraints are right: cross-document checks, inventory, link resolution against
the index and the banned-wording scans stay (okf-profile.md:56-59, 126-127); fixtures move,
none are dropped; the "standard library only" comments change. F-1 is the one hole: the task
as written has no entry point for the plain Markdown the test already judges.

**(f) The residual canonicalise-then-open race.** Documenting it is the right call under
SECURITY.md:37-44 ("running it on a checkout you would not build is out of scope"): the race
needs a concurrent writer inside the tree. `openat2(RESOLVE_BENEATH)` via `rustix` would close
it on Linux only, at the cost of a dependency and a second code path for other unixes, which
is a weaker and more complex guarantee than one documented race. The plan's design does
satisfy the sentence's real intent (file type on the opened handle, no stat before open).
F-6 asks that the Contract sentence be replaced rather than left beside a weaker Constraints
note.

**(g) Base-OKF keys.** The resolution follows the governing text and is not a pre-emptive
descope. AGENTS.md:49-51: "its capability documents in `docs/capabilities/` are the
specification." Architecture 22-26: "The closed key sets do not include the OKF base key
`resource`, so a document that carries it is rejected." okf-profile.md:123: "There is no
pass-through for extra keys." okf-profile.md:29-31: raf-okf "adds no rule of its own." No RAF
type has `tags` or `timestamp`, so "re-emitted sorted" has no subject, and a lenient mode would
be a second interpretation of the same documents. The refusal fixtures for `resource`, `tags`
and `timestamp` are the specification's own requirement (okf-profile.md:150-151), not a refusal
standing in for delivery; `title`/`description`/`type` are delivered as typed required keys.
The issue checkbox is contradicted by the specification, and the PR must say so with the
lines above (F-12) so the maintainer closes the issue knowingly.

## Completion rubric (Stage 1, plan only)

- UNFALSIFIABLE CRITERION: none found. Every acceptance check names an input, an entry point
  and an observed outcome; the thread-name check (S-28) is observed through the abort message
  on the shipped entry point.
- PRE-EMPTIVE DESCOPE: none. The declined enhancements each cite the governing document
  (lenient mode: okf-profile.md:123; cross-document checks: 56-59; openat2: trust model; YAML
  crate: comment keeping and `foldhash`), and the base-OKF decision is judged in (g).
- UNMAPPED REQUIREMENT: four. F-1 (okf-profile.md:20-23, 137 at `delivered` for plain
  Markdown), F-2 (okf-profile.md:104-115 Examples against the subset), F-3
  (source-index.md:70-72 recursion limit), F-4 (source-index.md:103-104 consumer
  documentation).
- PLANNED-BY-REFUSAL: none. The only refusals counted toward issue text are the ones the
  specification itself mandates (see (g)).

**Rubric result: FINDINGS-OPEN** (F-1, F-2, F-3, F-4 are required; F-5..F-12 are advisory).

VERDICT: NEEDS-REVISION

## Recheck (pass 2)

Revised plan re-read in full at the same path; the "Plan review dispositions" section maps
every finding. Only the new text each fix introduced was judged; cleared text was not
reopened. The one new edit outside the two capabilities (F-4) was checked against
`work/changes/workplan-document-model.md`: its `affects` lists
`cap:workplan-document-model#contract` (line 10), so the in-place Contract sentence needs no
record change.

| Finding | Status | Evidence in the revised plan |
|---|---|---|
| F-1 required | DISCHARGED | Task 2 (243-245): `raf_okf::Body::parse(text) -> Result<Body, ParseError>` over Markdown "with or without frontmatter", `Document::parse` builds its body through it, plain repository Markdown goes through it directly; Task 8 (356-359): `Body::parse` "for every frontmatter-less tracked Markdown file" and `docs_contract.rs` "keeps no link, fence or marker scanner of its own"; completeness row 174 with the fence/outside-link fixture and the PROPOSED-marker-still-a-block check; Task 2 docs name `Body::parse` in Contract and Usage (254-255). |
| F-2 required | DISCHARGED | Task 1 (219-220): "a flow mapping value is refused with a positioned error naming the key"; row 172: `bindings: {code: []}` fails at its line, neighbouring block form parses; Task 3 (269-271) and row 177: the Examples block rewritten to block-form `bindings:` and validated "as exact text", with the `resource:` variant naming the seven keys. |
| F-3 required | DISCHARGED | Row 182 and Task 6 (329-331): inline tables nested 1000 levels under the size cap make `Workspace::load` return a typed manifest error naming the file on the caller's 2 MiB stack; an 8-level manifest loads. Falsifiable either way: if the `toml` crate parses 1000 levels without error, the fixture fails and the crate must bound depth itself to keep source-index.md:70-72 true. |
| F-4 required | DISCHARGED | Task 9 (376-378): the sentence "A symbol that a macro generates is not in the index, so it cannot be bound." added to workplan-document-model Contract "Binding resolution"; row 191 cites both consumer documents and the docs-contract binding test refuses a `macro_rules!`-generated function with `NotFound`. The sentence carries no process wording. |
| F-5 advisory | DISCHARGED | Task 1 (228-229) and row 171: equality compares keys as a map with values and attached comments plus body bytes; stated in the Contract "Write" paragraph (234). |
| F-6 advisory | DISCHARGED | Resolved conflict 16 (122-126) names the replaced sentence and its replacement verbatim; Task 4 docs (288-289) carry it into Contract "Path containment". |
| F-7 advisory | DISCHARGED | Policy section (155-160): `cap` dev-dependency first, scoped allow only as the stated fallback with the reason naming why no safe path exists. |
| F-8 advisory | DISCHARGED | Policy section (140-147) and Task 5 (309-312): fmt, clippy and deny on `--manifest-path fuzz/Cargo.toml` with a crate-scoped NCSA exception, fixed `-seed`, pinned `cargo-fuzz` in CI, Makefile header and AGENTS.md gate list updated. |
| F-9 advisory | DISCHARGED | Policy section (148-151) and "Stage `Cargo.lock`" in Tasks 1, 4, 5, 6, 8. |
| F-10 advisory | DISCHARGED | Task 1 (226-228): `replace_span` rebuilds the body model and returns a parse error if the result is malformed; row 171. |
| F-11 advisory | DISCHARGED | Policy section (152-154); Task 9 (376) states unix-only in AGENTS.md section 2. |
| F-12 advisory | DISCHARGED | Task 10 (388-394) quotes the governing deny.toml, issue, architecture and okf-profile lines, the licence expression, the residual race and the platform. |

Two nits in the new text, neither material (advisory, fix in passing):

- Task 1 (216) still says `raf-okf` has "no dependencies" while the policy section (155-158)
  gives it the `cap` dev-dependency for the allocation fixture. Say "no dependencies; one
  dev-dependency, `cap`, for the allocation fixture", and run `cargo deny --offline check`
  in Task 1's focused validation since the graph changes there.
- The fuzz-manifest `cargo deny check` (141-143) must be `--offline` locally like the root
  `deny` target (the local machine blocks the advisory fetch, Makefile 34-38), and the plan
  should say where the NCSA exception lives: `--config deny.toml` against the root file
  (harmless to the root graph, which has no `libfuzzer-sys`) or a `fuzz/deny.toml`. Either
  is acceptable; name one so the CI job and the Makefile agree.

Completion rubric, pass 2: UNFALSIFIABLE CRITERION none; PRE-EMPTIVE DESCOPE none; UNMAPPED
REQUIREMENT none (F-1..F-4 discharged); PLANNED-BY-REFUSAL none.

**Rubric result: PASS**

VERDICT: PASS
