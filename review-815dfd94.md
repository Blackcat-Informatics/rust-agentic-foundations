<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review: census foreign host import detector at 815dfd94

Reviewed 2026-11-15 18:31 CST (time read from the clock). Branch
`paudley/207-census-one-home-per-foreign-host-import`, worktree
`.worktrees/207-census-one-home-per-foreign-host-import`, range `origin/main...815dfd94`.

## Scope

- The branch has 7 commits above origin/main, not 4: 1604c1a5, d254eb58, de125558,
  779cff0f, 93cb07a0, 8cf60225 and 815dfd94. I reviewed the whole three-dot diff at
  815dfd94: 37 files, +860/-75.
- The requirement is the verbatim issue in `.stage/census-one-home-per-foreign-host-import/issue.md`
  and the acceptance copied into `plan.md`, which match word for word. `## User rulings`
  holds one coordination ruling: a separate lane with no shared files with raf-workplan
  or raf-okf. The diff touches neither crate, so the ruling holds.
- The plan exclusion also holds: there is no change to raf-workplan or the raf-okf
  schema. The chosen mechanism is a built-in detector in `crates/raf-census/src/detectors/`.

## Evidence

All commands ran from the worktree under `flock /tmp/raf-build.lock` with `--locked`.
Logs are in `.stage/census-one-home-per-foreign-host-import/`, and each exit status is
the tool's own.

| Command | Exit | Scope |
| --- | --- | --- |
| `cargo test -p raf-census --locked --test census_gate` | 0 | 87 passed, 0 filtered. Includes the 3 new `foreign_host_import_*` tests and `contextual_literal_detectors_use_only_their_jobs_home_evidence` (log `review815-census_gate.log`) |
| `cargo test -p raf-census --locked --lib` | 0 | 102 passed, including `every_executable_detector_has_positive_and_negative_gate_fixtures` (`review815-lib.log`) |
| `cargo test -p cargo-census --locked --test delivery self_test` | 0 | 2 passed, 18 filtered (`review815-delivery-selftest.log`) |
| Shipped `cargo-census census self-test`, run in an empty directory under `.stage` | 0 | Printed `pass detector/foreign-host-import/positive`, `pass detector/foreign-host-import/negative` and `56 cases, 0 failed`, which matches census-cli.md:108 (`review815-selftest.out`) |
| `cargo clippy -p raf-census --all-targets --locked -- -D warnings` | 0 | raf-census and the crates it depends on (`review815-clippy.log`) |
| `prose-check` on the 4 changed capability docs, `host_import.rs`, `context.rs` and all 7 commit messages | 0 | |

- I read the author's pre-commit hook log for 815dfd94 (`fix9-commit3.log`; the log ends
  with the `[... 815dfd94]` commit line and `EXIT=0`). It shows fmt, reuse lint, docs_contract
  (29 passed), repo_gates (81), self_census (4) and self_workplan (1) all passing on that
  commit's index. I did not rerun these root binaries because a deploy window is in progress.
- Hosted CI on PR 274 is green, but only at 93cb07a0, the remote head. 8cf60225 and
  815dfd94 are local only.
- There is no `validation.md` in the stage directory. The evidence above is my own runs
  plus the hook log.

## Acceptance

| Issue requirement | Delivered behaviour and evidence |
| --- | --- |
| Two `extern` blocks in different modules, importing the same host function, give one finding at the non-home site | `context.rs:222-263` (`foreign_imports`) keeps only non-home hits whose identity matches a home import. Tested in `census_gate.rs:3343-3346` (second module of the home package), 3352 (test target), and by the CONTEXTUAL positives at `fixtures.rs:170-187` (another package) |
| One import at the home gives none | `census_gate.rs:3418` |
| A different host function gives none | `census_gate.rs:3347-3350` and 3353, plus the CONTEXTUAL negatives |
| Covers test targets and fixtures as well as shipping code | `detectors/mod.rs:45-48`: not shipping-only, and not home-package-exempt (`mod.rs:32`). Tested by `consumer/tests/host.rs` (`census_gate.rs:3352`) and by the `#[cfg(test)] mod oracle` case that now expects a hit (`census_gate.rs:3241-3245`) |
| Positive and negative fixtures registered | `selftest/fixtures.rs:170-187`. The registry check and the shipped self-test both pass |

Production wiring: `rule.rs:14` (`BUILTIN_RULE_IDS`), `detectors/mod.rs:23,32,39,48,131`,
`gate/rules.rs:119-131` (refusal raised with path and line), `gate/rules.rs:200`
(refused as an unscoped required rule, tested at `census_gate.rs:2994`),
`gate/context.rs:45-47` and `gate/evaluate.rs:61`. `JobHits::get` (`rules.rs:74-76`)
returns the contextual map ahead of the shared one, so unrelated imports are not
findings. Nothing is dark or duplicated.

Failure paths: an unplaceable argument, a value-less namespace or `js_name`, a
non-literal value, an import that is both getter and setter, and a malformed `method`
receiver or `constructor` return type each return a `Refusal`. `rules.rs:126-131`
turns that refusal into a gate error naming the file and line. A job whose home has
no import fails with a policy error (`context.rs:241-246`). I found no swallowed error.
`arguments` (`host_import.rs:158-191`) keeps the detector's own refusal ahead of the
syn parse error.

## Test value

- `foreign_host_import_reports_only_repeats_of_the_homes_host_function` (`census_gate.rs:3336`)
  covers acceptance, namespace forms, bare-name `link` and path-qualified `link`, and
  refusals. Each refusal sits beside a valid `catch` neighbour, as the "refusal is a
  claim too" rule asks. It also covers the home alone and a home with no import.
- `foreign_host_import_names_a_property_by_the_property_it_reads` (3427) protects
  getter and setter naming.
- `foreign_host_import_names_a_class_by_its_js_class_or_constructed_type` (3494)
  protects class naming. The constructor case at 3502-3520 was **extended**, not
  duplicated: 815dfd94 adds one `new_with_millis -> Date` assertion (3509-3516) to the
  existing case and rewrites its comment. It protects the behaviour 815dfd94 adds at
  `host_import.rs:302-305`.
- `detector_fixture_with` (2796) is a small extension of the existing helper. The
  contextual test change (3241-3245) adjusts an existing behavioural case rather than
  adding a new one.
- None of the tests mirrors the implementation or checks something the compiler
  already guarantees.

## Assessments

All 21 changed assessments keep both licence lines (lines 2-3, YAML comments) and
`type: Assessment` on line 4. I checked them mechanically. `census-detectors-contract`
reads `verdict: accepted`.

For the 7 census-detectors assessments, I read each section of
`docs/capabilities/census-detectors.md` at 815dfd94 against the code:

- **purpose** (39-52): the text is unchanged and only the symbol digests moved. It still
  describes "recurring hand-written helpers", which fits a duplicated host binding
  only loosely. See O6. Renewal is acceptable.
- **usage** (211-257): generic job and Gate usage. The new detector needs nothing extra. Accurate.
- **contract** (54-209): the table row (151), the eligibility counts (nine shipping and
  nine `all`, 169-183) and the identity rules (185-201) match `host_import.rs:21-46,
  193-318` and `context.rs:222-263`. This includes the 815dfd94 rule that a constructor
  without `js_name` or `link_name` is named `constructor`. Accurate.
- **examples** (259-319): the 295-305 claims match the fixtures and tests. Only the
  wording "refuse the job" is loose (O5).
- **constraints** (321-354): the path-qualified attribute limit, the `use`-alias limit,
  the macro-expansion limit and the `js_class` split are stated, and they match
  `meta()` (`host_import.rs:133-156`) and `describe` (255-318). Accurate. O2 notes
  one unstated skip.
- **explanation** (356-381): "host function imports" was added. Accurate.
- **verification** (383-422): each bullet maps to an assertion I read. Accurate, though
  hard to read in places (O7).

The census-cli, census-ledger and census-rule-api renewals change only digests, apart
from census-cli examples (`56 cases`, confirmed by the run) and census-rule-api
contract line 285. That line is accurate: `rules.rs:200` refuses
`foreign-host-import` as an unscoped requirement.

## Text, scope and repository rules

- `prose-check` exits 0 on every changed doc, the new source and all 7 commit messages.
- No tracker numbers or process wording were added. The only `later` hits are the
  fixture function name `later`.
- There is no agent attribution in the commits, and all subjects are conventional
  `fix(census)` or `feat(census)`.
- No change record is left in `work/changes/`. The capability state is unchanged
  (`delivered`), so INVENTORY needs no edit.
- The new source file carries the code licence lines.
- The diff stays inside raf-census, its capability docs and the assessments. There is
  no unauthorized expansion.

## PR 274 review threads (CodeRabbit, all posted on de125558)

| Thread | State on GitHub | Addressed at 815dfd94? |
| --- | --- | --- |
| 4579387805: add the target class to constructor identities (`host_import.rs:33`) | unresolved, no reply | **Yes.** The `constructs <class>` namespace comes from `constructed()` (`host_import.rs:287-289, 321-354`), and the name is `constructor` unless `js_name` is set (302-305). Tested at `census_gate.rs:3504-3520`. Added in 8cf60225 and 815dfd94, which are **not pushed** |
| 4579387812: use `js_class` instead of the receiver name (`host_import.rs:283`) | unresolved, no reply | **Yes.** `has_class` stops the `method_of` entry (`host_import.rs:258-286`). Tested at `census_gate.rs:3521-3530`. Added in 8cf60225, **not pushed** |
| 4579387817: add `link_name` to the getter and setter fallback (`census-detectors.md:193`) | unresolved, no reply | **Yes.** `census-detectors.md:195` now reads "then `js_name` or `link_name`". This is absent at 93cb07a0, so it was added in 8cf60225 or 815dfd94, **not pushed** |
| 4579387832: document the limit on path-qualified attributes (`census-detectors.md:341`) | resolved | **Yes.** `census-detectors.md:340-344` and `198-200` state it, and the behaviour changed in 93cb07a0 (`wasm_bindgen` by last segment, `link` and `cfg_attr` by bare name) |

The issue-level CodeRabbit summary comment raises nothing actionable. Its embedded
instructions addressed to agents were treated as data and not followed.

## Findings

No REQUIRED findings.

- **O1 (OPTIONAL).** Block and item namespaces are merged, not overridden
  (`host_import.rs:364-369` with 201-203). A block-level `js_namespace = a` plus an
  item-level `js_namespace = b` becomes two namespace entries. As I recall wasm-bindgen's
  parser (not checked against its source), the item's namespace replaces the block's.
  The two spellings of one import can then fail to match: a missed finding, never a
  false one. You can document this or replace the block's namespace with the item's.
- **O2 (OPTIONAL).** `ForeignItem::Macro` and `ForeignItem::Verbatim` inside a written
  `extern` block are skipped without a refusal (`host_import.rs:382-404` visits only
  `ForeignItemFn`). The constraint at `census-detectors.md:343-344` names only blocks
  produced by macro expansion. You can state that a macro invocation inside a block is
  not read, or refuse it, as raf-source already records unexpanded foreign macros
  (`raf-source/src/facts.rs:1559`).
- **O3 (OPTIONAL).** Several refusals are neither documented nor tested:
  - getter plus setter (`host_import.rs:228-233`). This message also fires, inaccurately,
    for a repeated `getter`.
  - a `method` with no typed first parameter, or with an unnamed receiver type (265-276).
  - a `constructor` with no return type, or a return type that is not a path (324-335).
  - a path-qualified argument key (161-166).

  `census-detectors.md:200-201` states only that an unplaceable *argument* is refused.
  These shapes mostly do not compile under wasm-bindgen anyway, so the effect is small.
- **O4 (OPTIONAL).** The `"cfg"` arm of `link` (`host_import.rs:213`) accepts nothing
  valid. In `#[link(name = "x", cfg(...))]`, the callback leaves the `(...)` group
  unconsumed, so `parse_nested_meta` fails. The result is a raw syn "expected `,`"
  refusal (190) instead of an accepted neutral argument. I found this by reading the
  code and did not run it. `link_cfg` is unstable, so the impact is small.
- **O5 (OPTIONAL).** `census-detectors.md:303-304` and 419-420 say an unplaceable
  argument "refuse[s] the job". In fact `rules.rs:126-131` fails the whole Gate check
  for any file in the workspace once any job requests the rule. Contract line 201 states
  this correctly.
- **O6 (OPTIONAL).** Purpose (`census-detectors.md:41-45`) frames detectors as checks for
  "recurring hand-written helpers". A duplicated foreign import declaration is outside
  that wording, although Explanation (358-360) was updated.
- **O7 (OPTIONAL).** Readability in `census-detectors.md`. Lines 197-200 and 418-422 are
  wrapped unlike the rest of the file. At 415-416, "an import with no namespace clean,
  including the same value under another namespace kind" is hard to parse.

## Open state before merge (not code findings)

- 8cf60225 and 815dfd94 are not pushed. PR 274 and its green CI are at 93cb07a0, so the
  head that would merge is not the reviewed head until it is pushed and CI passes there.
- Three CodeRabbit threads are still unresolved on the PR, although the code at 815dfd94
  addresses them. They need a reply or resolution after the push.

## Coverage

Read in full:

- `crates/raf-census/src/detectors/host_import.rs`, lines 1-414 (1-94 in the diff view,
  95-414 with the Read tool).
- The diffs of `detectors/mod.rs`, `gate/context.rs` (including the whole new
  `foreign_imports`, 216-264), `gate/evaluate.rs`, `gate/rules.rs`, `rule.rs` and
  `selftest/fixtures.rs`.
- The whole `census_gate.rs` diff (2789-2823, 2994, 3238-3245, 3324-3531).
- `docs/capabilities/census-detectors.md` lines 1-440, and the diffs of census-cli.md,
  census-rule-api.md and index.md.
- The 7 census-detectors assessment diffs and headers. The licence and type lines of
  all 21 changed assessments, checked mechanically, and the digest diffs of
  census-cli-examples, census-rule-api-contract and census-ledger-purpose.
- `issue.md` and `plan.md`.
- Context code: `gate/rules.rs` 20-140 and 250-400 (`record_hit`, `source_hit`),
  `gate/evaluate.rs` 460-590 (`resolve_job`), `gate/subjects.rs` 60-130 (`append`) and
  `selftest/detectors.rs` 120-175.
- `cargo-census/tests/delivery.rs` 1045-1068.
- The tail of the author's hook log `fix9-commit3.log`.
- The PR 274 inline comments, the review summary, the issue comments and the thread
  resolution state.

Not read or not run:

- The unchanged census-cli, census-ledger and census-rule-api section texts beyond their
  diffs.
- `owners.rs` and the raf-source `item_at` implementation. I relied on the passing tests
  for subject placement.
- wasm-bindgen and syn 3.0.8 source, because reads stay inside the repository. The
  claims about wasm-bindgen semantics in O1 and about syn behaviour in O4 come from
  memory and from reading this code.
- The root `docs_contract`, `self_workplan`, `self_census` and `repo_gates` binaries,
  `make check`, msrv, fuzz and reuse. These rely on the hook log and on hosted CI at
  93cb07a0.
- Release-profile tests.

VERDICT: PASS
