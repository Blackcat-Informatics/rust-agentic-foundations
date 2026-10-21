<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Validation index: raf-census core, normal form 1

## Current combined qualification

C7 is committed/pushed as 3669223 after independent review and the full normal pre-commit hook. The landed source-root delivery 34dac35 was integrated by ordinary base-to-issue merge 0d2e16c, also committed/pushed with the full normal hook. The only conflict was a generated source-contract section digest: both root discovery and one-parse parameter/initializer facts remain in the merged document and symbol inputs; fresh context was read and its assessment regenerated with SPDX. All other assessments were current. `make docs-sync` exited 0. The commit hook passed docs_contract 28, repo_gates 68 and self_workplan 1 on the combined inputs.

Mandatory `make check` PASSED on 0d2e16c, terminal exit 0: raw/queue-combined-make-check.log. This covers fmt, workspace clippy/tests, source release tests, rustdoc, deny, REUSE, MSRV workspace/fuzz checks, all five live fuzz targets at 5000 runs each, Apple/FreeBSD/Windows source cross-checks and repository gates. All 12 hosted jobs in run 39540739423 passed, including native macOS/Windows, for PR head 0d2e16c; pr-checks.txt retains the job links. PR 39 references issue 3 without closing it: full census WASM acceptance stays open through 46.

Final integration assessment: refreshed actual base is34dac35a31a78b21e74fb4a00fb07355e1e77881; head is0d2e16c309b40693b13082d79a1e4061965038c6. `git merge-tree --write-tree --no-messages <base> <head>` exited0 and produced e8411c50dbd489c2413f33920362c8e0c0c4f879. Diff of that candidate against the tested head exited0 with no differences. The base is already integrated in the tested branch, so the combined full gate and all hosted results cover the actual candidate tree. No temporary integration checkout or redundant synchronization is needed. Before invoking ghprsq, native paginated review and inline captures again exited0 with zero entries; GraphQL shows zero threads and hasNextPage false. Discussion capture completed, with no new substantive feedback. Final source working tree is clean; all selected evidence writers stop before capture. The helper refetches internally, so actual landing inputs/tree must still be checked afterward rather than promising an expected-input lock.

Evidence packaging: tasks/queue-archive-inventory.md inventories the historical review clones and RDF comparison copies. Eight unique probes and eleven variant overlays were preserved raw and compared byte-for-byte before removing redundant source copies and Git stores. Exact bases and reconstruction instructions are scratch/replay/README.md and clones.tsv. Independent oracle programs/samples, measured sources, logs and fuzz inputs remain raw. Shared cache destinations were untouched; stage is 17 MiB with no symlinks. This changes packaging only, not behavior qualification.

## Current C7 acceptance and handoff

C6 is committed and pushed as `29df7f6`. C7 is implemented and ready for the parent to stage, commit and push normally. Independent substantive review and generic-callee repair recheck PASS: `tasks/queue-c7-review.md`. No C7 commit, push, full workspace check or merge has been performed by this implementation lane.

* Full 256-bit BLAKE3 body/shim identity, strict 64-hex machine spelling, separate human abbreviation, one fingerprint representation per print, escaped parser diagnostics and regenerated canonical RDF/parser pins.
* Counted policy rationale and exact RDF rows: `docs/census-selection.md`, `docs/data/census-selection.ttl`. Post-review source freeze: `raw/selection-measured-source.patch`, `scratch/selection-measured.rs`; final run `raw/selection-counts.ttl/log`, exit 0. Source census: 417 bodies / 50,664 tokens; all source corpora: 9,009 selected, 155 body groups + 49 shim groups, 393 pairs. Explicit synthetic-label and macro limits remain.
* Shared generic-callee repair: associated function types stay eligible, lifetime/function-pointer Self calls are excluded; direct query and loaded neighbors pass, and the included Rust fixture separately compiles with warnings denied. Earlier failed and interrupted runs remain historical evidence, not passes.
* Actual fresh lifecycle gate: `cargo workplan change complete --root . census-identity-and-selection`, exit 0, removed its record (`raw/identity-self-complete.log`). Five implemented outcomes and seven affected sections are retained in `scratch/identity-change-before-complete.md`. All seven assessments are accepted/current with SPDX. The new Explanation record is reflected by the sorted inventory and its exact counts (63 assessments, 65 total records).
* Current focused gates: census all-target clippy and all 133 tests pass (`raw/identity-self-clippy.log`, `raw/identity-self-census.log`); current fuzz all-target clippy passes (`raw/identity-self-fuzz-clippy.log`). Root docs_contract 28, repo_gates 68, self_workplan 1 pass, terminal exit 0 (`raw/identity-root-gates-current.log`). The earlier missing inventory-count failure is corrected, preserved and superseded.
* Isolated terminal `make deny` and `make reuse`: exit 0 (`raw/identity-deny-terminal.log`, `raw/identity-reuse-terminal.log`); all 623 files licensed. Independent report parser pins: 41/89/11 triples (`raw/identity-oracle.nt/log`); selected count RDF independently parses to 3,588 triples (`raw/identity-counts-oracle.nt/log`).
* BLAKE3 explicitly disables default features and enables pure. Exact 1.8.7 no_std hash-only wasm32 probe passes (`raw/identity-hash-wasm-probe.log`). This does not prove full source/census wasm builds. Full #3 acceptance remains open through tracked #46; the PR must not close #3.

`raw/identity-workplan-check-final.log` is historical pre-completion evidence: it records the delivered/open-record refusal and is not cited as a current pass. Post-completion currentness is verified by the successful completion re-evaluation and the passing root self_workplan test; no duplicate check is needed.

Next owned delivery steps: parent normal commit/push, synchronize cleanly with landed origin/main source-root delivery, preserve both source contracts, then run the mandatory full combined-tree gate and hosted qualification before ghprsq landing. No full gate duplication was started here. Detailed evidence and scope: `tasks/identity-selection-evidence.md`, `acceptance-gaps-current.md`.

## Historical evidence below — earlier scopes and states are superseded

The following sections preserve their original observations. Statements that C6 is uncommitted or that identity/corpus work is still unimplemented describe those earlier snapshots and are superseded by the current status above.

## C6 RDF report implementation, awaiting review/commit handoff

The uncommitted C6 replacement removes census serde/serde_json, uses the shared
purrdf-lex 3.0.1 caret floor with the same scoped audited Unicode exception in
both deny files, writes/reads canonical ground Turtle 1.2, preserves normalForm
on reads while gating reportForm, and names typed match reifiers/triple terms.
Unit identity includes symbol/file/line/startByte, including same-line cfg
alternatives. The normative shape table, real Examples, downstream designed
CLI/migration/debt/ledger text, reader fuzz wiring and security text are updated.

* Real baseline example compiled/rendered; raw/queue-c6-example.txt and .log.
* All-target census clippy passed; raw/queue-c6-clippy.log. Initial focused run:
  21 library, 7 page and 23 report tests passed; raw/queue-c6-reports.log.
  After diagnostic hardening and original-graph reader rendering: 24 report tests
  passed, plus six profile tests; raw/queue-c6-reports-final.log and
  raw/queue-c6-profile.log. The additional canonical-line refusal test passed
  separately; raw/queue-c6-canonical-lines.log. No zero-test acceptance.
* Independent purrdf-rdf 3.0.1 parser accepted the exact baseline, index and
  awkward-name Turtle without a caller base (41/89/11 triples). Its sorted
  N-Triples expansions are pinned and compared with this reader's graph.
  Exact command/source/input/output inventory: raw/queue-c6-oracle-command.txt.
* Report-reader fuzz target check and fuzz-harness clippy passed;
  raw/queue-c6-fuzz-check.log and raw/queue-c6-fuzz-clippy.log. Live target ran
  5000 runs, exit 0; raw/queue-c6-fuzz-run.log. Generated mutations stay under
  stage. The run precedes the final allocation/diagnostic review refinements;
  final mandatory make check must exercise the settled code.
* make deny and make reuse passed; raw/queue-c6-deny.log and
  raw/queue-c6-reuse.log. Both licensing files keep the same audited exception;
  canonical fixture paths are explicitly annotated in REUSE.toml.
* docs-sync passed; all six census assessments were read/refreshed with SPDX
  options. Root document/repository/self-workplan tests passed after staging
  the module-directory replacement; raw/queue-c6-root-gates.log. Final document
  recheck exited 0 (docs_contract 28/28, self_workplan 1/1) in
  raw/queue-c6-document-recheck.log. The preceding workplan check also exited 0
  with no findings; raw/queue-c6-workplan-check.log. Final all-target clippy after
  the canonical-line fix exited 0; raw/queue-c6-clippy.log.

Review fix details and exact coverage limits: tasks/queue-c6-fix-evidence.md.
No full make check, commit, push or merge has been performed for C6. The wider
current #3 contract remains open for full-width fingerprints, counted corpus
threshold justification and the mapped pure-core/portability work; see
acceptance-gaps-current.md. C6 is not whole-issue completion.

## Current queue acceptance update

C4/C5 committed as b1ba90e: census-normalize fuzz target and named seeds, actual
make-fuzz/CI caller wiring, BLAKE3 caret floor, security/parse description,
source-index dependency, ledger/migration requirement repairs and explicit
caller-span invalidation warning. No remaining C4/C5 behavior is knowingly left
open; final full qualification is still required after C6.

* Final live target: `cargo fuzz run --no-cfg-fuzzing census-normalize
  .stage/raf-census-core-normal-form-v1/raw/c4-fuzz-current
  fuzz/corpus/census-normalize -- -runs=5000 -seed=1 -max_len=400000
  -rss_limit_mb=2048 -malloc_limit_mb=512`: exit 0, 5000 runs;
  raw/queue-c4-fuzz-current.log. This includes the stronger source-facts self-check.
* `cargo clippy --manifest-path fuzz/Cargo.toml --locked -- -D warnings`: exit 0;
  raw/queue-c4-fuzz-clippy.log. `make deny`: exit 0, advisories/bans/licenses/sources
  all pass; raw/queue-c45-deny.log.
* Root document/repository/self-workplan gates: exit 0;
  raw/queue-c45-gates.log. docs-sync exited 0, constraints context read and
  reaccepted with SPDX options. The other assessments remained current.
* Independent review tasks/queue-c45-review.md required the caller-span warning;
  tasks/queue-c45-recheck.md PASS after that fix and the stronger fuzz self-check.
  Normal commit hook passed; raw/queue-c45-commit.log.

C3 source-consistency and target-kind remediation committed as fe26b39. GAP-14
and GAP-17 are implemented; GAP-12 reader-version checking and GAP-13 serde_json
removal remain required in C6. The remaining C4-C6 requirements are not complete.

* `cargo test --locked -p raf-census --test units`: exit 0, 25 passed, zero
  failed/ignored/filtered, including Lib/Test aggregation for a shared file;
  raw/queue-c3-units.log.
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`:
  exit 0; raw/queue-c3-gates.log. Focused qualification, not a full make-check claim.
* Independent review tasks/queue-c3-review.md found missing SPDX headers in six
  refreshed assessments; all restored. tasks/queue-c3-recheck.md PASS covers the
  fix and the shared-file assertion.
* Normal signed commit and full configured pre-commit hook passed;
  raw/queue-c3-commit.log. No hook bypass. Full `make check` remains required after
  the remaining census fixes settle; current green hosted evidence is historical
  C2 and does not qualify C3 or unfinished RDF output.

Branch paudley/3-raf-census-core-normal-form-v1, base origin/main (c5fc94f). Plan: plan.md (Rev 2 authoritative).
Toolchain: floating nightly (rust-toolchain.toml); MSRV 1.98.1 via `make msrv` (T6, exit 0).

## Commits and task verdicts
| Task | Commits | Review | Final verdict |
|---|---|---|---|
| T1 scaffold, options, errors, version gate, records | cb08b0c | tasks/T1-review.md | PASS |
| T2 normaliser, fingerprints, goldens | 08071b8, 3f8f469 | T2-review BLOCKED → T2-recheck | PASS |
| T3 units, Census::load | 05d7be1, e1073f9 | T3-review BLOCKED → T3-recheck | PASS |
| T4 grouping, tables, constants, jobs | e9ea57e, e02a444 | T4-review BLOCKED → T4-recheck | PASS |
| T5 reports, example, determinism | a8452e5, 2ec6a1e | T5-review BLOCKED → T5-recheck | PASS |
| T6 documents, completion | 7a49256, ab62795, be93118 | T6-review BLOCKED → T6-recheck (also the pre-PR completeness review, R1-R18 table) | PASS |

## Focused checks at be93118 (T6 report and T6 recheck)
* `cargo test -p raf-census --locked`: exit 0 (22 unit, 20 forms, 4 golden, 16 groups, 5 page, 11 reports, 20 units, 1 doctest).
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: exit 0 (28/68/1).
* `make fmt clippy doc deny reuse issue-refs`: exit 0. `make msrv`: exit 0 (at 7a49256; ab62795/be93118 change one pass and docs, no dependency or MSRV-relevant surface).
* `cargo workplan --root . check`: exit 0. `change complete census-normal-form`: exit 0 at 7a49256.
* `cargo run -p raf-census --example baseline` twice, with and without `-- crates`, absolute and relative: byte-identical.

## Full qualification
The workflow's single full `make check` is pending: it runs after Stage 2 fixes settle.

## Not run / limits
* No timing runs, no mutation passes (repository rule).
* Hosted CI: pending on the PR.

## User decision (Stage 2)
The user decided, verbatim: "Note - breaking the API is encouraged". Consequence: API shape concerns (open structs, future fields, report shapes) are judged on the right design now, not on compatibility; a later breaking change is acceptable.

## User decision (Stage 2, GAP-07)
Question: how to handle the pins that the normaliser fixes change, given that normal form 1 has never been merged and CLAUDE.md forbids changing what a fingerprint hashes without a bump. The user answered, verbatim: "Defined at first merge (Recommended)" (the option text: keep NORMAL_FORM = 1, regenerate the pins; CONTRIBUTING.md and AGENTS.md state that a version is fixed at its first merge to the default branch and any later change is a new version; the user adjusts CLAUDE.md's wording).

## Standing goals (.goals, untracked in the main checkout, read 2026-10-20)
GREENFIELD-FIRST; RUST-FIRST, PYTHON-SURFACE; SUBSUME, EXTEND, ENHANCE; LOW/NO OPTIONALITY, HARD FAILS (build); RDF 1.2 ONLY ("If you find yourself using JSON you have failed"); MAXIMAL UTILITY, PERFORMANCE, PORTABILITY (wasmable core).

## User decisions on .goals (verbatim)
* Report format question → "RDF 1.2 reports (Recommended)" (option text: replace the JSON reports with RDF 1.2 (Turtle 1.2 text, with triple terms where needed), deterministic and round-tripped through a strict reader; drop serde_json from raf-census; census-cli and migration designs get updated to match).
* Scope of .goals over existing work → "file comments on any known open issues, file issues for anything not covered by that."
## Historical C6 queue snapshot (superseded)

C6 RDF report implementation was committed and pushed as `29df7f6` after independent review/recheck and the normal hook (docs_contract 28, repo_gates 68, self_workplan 1, all passing). Raw terminal results: `raw/queue-c6-commit.log`, `raw/queue-c6-push.log`. The final original-graph/diagnostic changes have focused checks; the 5000-run reader fuzz result is from the preceding implementation, as detailed in `tasks/queue-c6-fix-evidence.md`.

The full issue is not complete. `acceptance-gaps-current.md` maps the newer full-identity/corpus/portable-boundary requirements; that implementation is now underway in this checkout. Mandatory final make check, current Linux CI, integration and merge remain pending. Earlier sections below are historical qualification and must be read with this current state.

