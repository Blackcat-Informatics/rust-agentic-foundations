# Stage 2 remediation plan: PR #39 (issue 3, raf-census core)

Inputs: gap-analysis.md (17 gaps), completion-audit.md (F1-F3, FINDINGS-OPEN), historical-fit.md.
Review path: full (new crate, novel normaliser, shared contract with six designed census capabilities).
User decision, verbatim: "Note - breaking the API is encouraged".

## Decision on GAP-07 (version of the pins)
Normal form 1 has never been merged. No ledger, debt record or release holds a v1 fingerprint, and
`origin/main` has no census code. The rule exists to keep ledgers and recorded debt valid, and that
purpose is not engaged. Decision: normal form 1 is defined by the content of its first merge; the pins
are regenerated once in C1, with `NORMAL_FORM = 1`. CONTRIBUTING.md states the rule (a version is
fixed at its first merge to the default branch; after that, any change that can move a fingerprint
is a new version). The PR quotes the decision and asks the maintainer to confirm it with the other
open items. If the maintainer refuses it, the fix is a one-line bump plus a pin regeneration.

## Gaps, fixes, checks and commit boundaries

### C1: normaliser corrections (one commit; the goldens regenerated once). GAP-01, 02, 03, 04
* **GAP-01** rename only bound names. A name is renamed only if the signature or the body binds it:
  parameters, and names bound by `let`, closure parameters, `for`, `match` arms, `if let` and
  `while let` patterns, labels, lifetimes and local type declarations. Every other identifier is kept.
  Check: registry pairs `map(double)`/`map(triple)` Distinct; two lowercase statics Distinct;
  renamed closure params and `let` bindings Equal(Value); a `Census::load` test in `tests/groups.rs`
  showing the two different-function bodies in no common group beside a renamed-local pair that groups.
* **GAP-02** the block after an `if`/`else if`/`while`/`match` scrutinee/`for` head is never a struct
  literal. Check: `clamp_a`/`clamp_b` (`if next == MAX { next }`) Equal(Value) beside a real struct
  literal `S { x }` kept.
* **GAP-03** parameters and locals are numbered together by first appearance, from the literal-tier
  level up. The shim level keeps signature order. Check: `count_a(xs, k)`/`count_b(k, xs)` with
  byte-identical bodies Equal(Value). The forwarders `fn a(x, y) { f(y, x) }` and
  `fn c(x, y) { f(x, y) }` are Distinct at the shim level.
* **GAP-04** every threshold (literal tier, `min_tokens`, shim bounds) counts the length of the
  value-level form, decided before `kind_literals`. Check: the tier-straddle pair from the gap
  analysis is Equal(Value). For the `min_tokens` straddle, both bodies have a body print or neither
  does, at the default and at 10. `tests/page.rs` asserts the counting rule.
* All four: the goldens and `examples/goldens.rs` are regenerated once, and the monotonicity test
  passes. Focused check: `cargo test -p raf-census --locked`.

### C2: parse on the budgeted stack. GAP-05
* `units::collect` (all `proc_macro2`/`syn` work of a load) runs on a scoped thread whose stack is
  raf-source's parse stack size (`PARSE_STACK_BYTES`; made public in raf-source if it is not, with
  its doc). A panic or error on that thread is an error of the load, never a fallback.
* Check: a `units.rs` test runs `Census::load` inside a thread built with a 2 MiB stack. The 8 × 60
  signature returns `Ok` with one unit. The 24 × 60 signature returns `Error::Load` naming the
  recursion budget. The Constraint and the page's Token source state the bound.

### C3: code corrections. GAP-12, 13, 14, 17
* **GAP-12** both `from_json` docs and the page's Reports section say reading does not gate the
  version. A test reads a report with `normal_form: 7` and shows `check_normal_form` refuses it.
* **GAP-13** `render` calls `Value::sort_all_objects()`, and the `serde_json` floor is raised to the
  first release that has it. The sorted-keys and Examples goldens pass.
* **GAP-14** an unresolvable item id or an empty reach list is `Error::Source` naming the file. No
  `filter_map`/`continue` skip remains.
* **GAP-17** `FileCtx` carries the target kinds that reach the file (raf-source `TargetKind`) beside
  `shipping`. A test asserts the kinds of a lib, a test and an example file.

### C4: dependency and security surface. GAP-09, 10
* **GAP-09** `blake3 = "1.8.7"` (a caret floor; excludes 1.8.5 and its BSD dependency). Check:
  `make deny`.
* **GAP-10** SECURITY.md names the census's parse surface: what it reads, its bounds (stack, file
  size) and its failure mode. The fuzz harness (`fuzz/`, its own workspace) gains a target that
  runs `Normalizer` over arbitrary text and `Census::load` over a one-file `MemoryTree`, with a
  committed seed corpus. Check: `make fuzz` (fmt, clippy and deny of the harness, plus the corpus
  runs).

### C5: documents and records. GAP-06, 07, 08, 15, 16, plus the text of C1-C4
* **GAP-06** `work/changes/census-ledger.md` gains two items. (a) Matches of `forbidden.names`,
  `forbidden.patterns` and `rule:` hits are classified HOME, VARIANT or COPY and counted with the
  fingerprint and constant matches, each unit counted once. (b) Home, entry-point and variant paths
  resolve through re-exports to unit symbols before jobs are matched. The record's description is
  corrected. `cap:census-ledger`'s classification paragraph says fingerprint and constant matching
  belongs to `cap:census-normal-form` and that the ledger extends it.
* **GAP-07** the CONTRIBUTING.md rule above.
* **GAP-08** `cap:census-normal-form` has `depends: [cap:source-index]`.
* **GAP-15** AGENTS.md and CONTRIBUTING.md say "when the census `Gate` is built".
* **GAP-16** `work/changes/census-migration.md` gains an item: the index read is bounded so that it
  admits the index the census writes (bounded by the census's load budget, not the ledger's 1 MiB).
* The page and the capability state C1-C4 (rename rule, first-appearance numbering, threshold
  count, stack bound, version-free reading, `FileCtx` kinds). The Examples are re-pasted from the
  real example output, and the affected assessments are re-read and accepted. Checks:
  `cargo workplan --root . check`, `docs_contract`/`self_workplan`/`repo_gates`, `make docs-sync`
  (no change on a second run), `make reuse issue-refs`.

### GAP-11: qualification
After C1-C5 are committed and pushed: the workflow's single full `make check`, run in the background
on the head, recorded in validation.md. Hosted CI on the new head, polled by an agent.

## Execution
Each commit: an implementer (sonnet/medium) with the NO DEFERRALS clause and the no-timing,
no-mutation rule. Then an independent fix review (`tasks/GN-review.md`, sonnet/medium), fixes and a
recheck, then push and a comment on issue and PR. Order: C1, C2, C3, C4, C5 (C5 documents the
behaviour of C1-C4). The exit gate is a fresh completion audit (opus/xhigh).

## Rev 1: integration of remediation-plan-review.md (authoritative over the text above)
* **GAP-07 decided by the user before C1**, verbatim: "Defined at first merge (Recommended)" (recorded in validation.md with the option text). C1 rewrites CONTRIBUTING.md:23-27 and AGENTS.md:237 together into one rule: a normal form version is fixed by the content of its first merge to the default branch; after that, any change that can move a fingerprint is a new version with a migration path. The "Never change normal form 1 in place" sentence becomes that rule. CLAUDE.md is not edited by an agent: the PR and the final summary tell the user that its "Never ... without bumping `normal_form`" line needs the same qualifier.
* **C1 holds everything its own checks read (review B2):** the page (`docs/architecture/normal-form.md`) and the capability text for the rename rule, first-appearance numbering with the Decision restored to "by order of first appearance", and the value-form threshold count. It also holds the Examples re-pasted from the real example output and the affected assessments re-read and accepted. C1's checks add `cargo workplan --root . check` and `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`. C5 keeps only GAP-06, 08, 15 and 16 and the text of C2-C4.
* **GAP-01 binder coverage (review B3):** each binder form gets one registry pair whose two sides bind different names and are Equal(Value). The forms are `let`, `let` with a tuple or struct pattern, closure parameters, `for`, a `match` arm binding, `if let`, `while let`, a label, a lifetime and a local `struct`/`fn` declaration. The rule as stated: every use of a bound name in the body is renamed, including call position (`let f = |x| ..; f(v)`) and a name that shadows an item. A fixture pins `let f = ..; f(v)` vs `let g = ..; g(v)` as Equal(Value).
* **GAP-05 (advisory):** a raf-source helper runs a closure on the parse stack and re-raises a panic with `resume_unwind`, as `parse_source` does. The census calls it, so the policy lives in one place. `PARSE_STACK_BYTES` is already public.
* **GAP-02 (advisory):** the gap's `Census::load` test is kept, and `S { x }` vs `S { y }` is a registered Distinct pair (field names kept).
* **GAP-10 (advisory):** the corpus includes the GAP-05 8 × 60 reproducer, and CONTRIBUTING.md:70-72 lists the new fuzz target.
* **GAP-14 (advisory):** the check is that no unresolvable id or empty reach is skipped. The `unit_kind` filter of kinds that yield no unit (units/mod.rs:137-139) is kept.
* **GAP-04 (advisory):** `Tower::structural_tokens` is renamed to the value-level count, and the threshold-constant, `Print`, `Group::tokens` and index `tokens` docs are corrected.
* **GAP-09 (advisory):** the manifest comment no longer says "exact version". `make deny` and `cargo tree -i arrayref` (no match) are the checks.
* **GAP-11 (advisory):** pass criteria are `make check` exiting 0 on the pushed head and all twelve hosted jobs reporting SUCCESS. A macOS or Windows-only failure is reported and does not block, per CLAUDE.md's platform policy.

## Rev 2: plan-review pass 2 advisories, .goals and the RDF decision (authoritative)
* **N1:** pairs that differ only in bound names are expected `Equal(LiteralTier)`, the exact level at which they first become equal: the binder pairs, `f(v)`/`g(v)`, `clamp_a`/`clamp_b` and `count_a`/`count_b`. Only the GAP-04 tier-straddle pair is `Equal(Value)`. The registry check is not loosened.
* **N2:** a name in field position, including struct shorthand (`S { x }`), is kept as a field name even when it is also a bound local. The `S { x }` vs `S { y }` pair is Distinct.
* **N3 / GAP-05, revised for .goals (maximal performance, wasmable core):** the preferred fix is one parse. The census takes parameter bindings and initialiser ranges from raf-source's own parse facts (adding the facts raf-source must expose, stated in `cap:source-index`'s Contract, docs-synced, with assessments re-accepted). Signatures and items are then not re-parsed with `syn` on the caller's stack. Body lexing with `proc_macro2` stays, and its depth is bounded by raf-source's delimiter budget, which a 2 MiB-thread test proves. The fallback, only if the facts cannot be exposed, is the raf-source parse-stack helper of Rev 1, stated in `cap:source-index`. The acceptance checks of GAP-05 are unchanged.
* **N4:** GAP-11 pass criteria are `make check` exiting 0 and every Linux hosted job reporting SUCCESS. A macOS- or Windows-only failure is reported and does not block.
* **RDF 1.2 reports (user decision, verbatim "RDF 1.2 reports (Recommended)"):** a new commit **C6** replaces the JSON baseline and index with RDF 1.2 reports per `rdf-reports-design.md` (to be written and reviewed). It drops `serde_json` from raf-census, re-pastes the Examples, updates the page's Reports section and `tests/page.rs`, and edits the designed downstream capabilities named there. C1 still re-pastes the JSON Examples so that its own checks pass; C6 replaces them.
* **.goals elsewhere (user decision, verbatim "file comments on any known open issues, file issues for anything not covered by that."):** handled outside this PR from `.stage/goals-survey.md`.

## Rev 3: RDF 1.2 reports (C6), per rdf-reports-design.md
* C6 implements `rdf-reports-design.md`: `purrdf-lex =3.0.1` is the only new dependency (plus the pinned `Unicode-3.0` exception, identical in `deny.toml` and `fuzz/deny.toml`); `serde`/`serde_json` are removed from raf-census; Turtle 1.2 writer and closed-shape strict reader in raf-census; the vocabulary, IRI scheme, hashed `BASE`, triple-term job matches, `census:normalForm` and `census:reportForm`; the replacement list and downstream doc edits of its sections 5-7 (cli, migration, debt-ratchet `census-debt.ttl`, ledger constraint).
* BLAKE3 stays the `blake3` crate, as the issue names it; a second BLAKE3 (`purrdf-hash`) comes in transitively through `purrdf-lex`. Collapsing the two is recorded on the new-issue tracker for machine output (comment on the JSON→RDF issue), not done here.
* Checks: every negative fixture of the JSON reader has its RDF counterpart and a valid neighbour; two runs byte-identical; the reports parse in an independent RDF 1.2 parser (purrdf-rdf in a scratch crate, as the design did); `cargo check -p raf-census --target wasm32-unknown-unknown` is NOT a C6 check because raf-source does not build for 32-bit targets (comment filed on the wasm-gate issue).

## Rev 4: plan-review pass 3 (authoritative over Rev 3 and the design where they differ)
* **B4:** `purrdf-lex = "3.0.1"` (caret floor). The page states the term escaping as the RDF 1.2 canonical N-Triples form, and an awkward-names golden pins its exact bytes, so a purrdf-lex release that changed escaping fails the golden rather than silently moving output.
* **B5:** C6 adds a fuzz target `census_report_read` that reads arbitrary text with the strict reader and asserts that any accepted text re-renders byte for byte. It is seeded with each refused construct and one valid report per kind. SECURITY.md (the census parse surface section C4 writes) is updated to the Turtle reader. CONTRIBUTING's fuzz-target list and `fuzz/Cargo.lock` change in the same commit.
* **P1:** the match reifier is typed `census:Match`. Every subject has exactly one `rdf:type`.
* **P2:** a unit IRI key is symbol, file, line and start byte. A fixture puts two cfg alternatives of one symbol on one line.
* **P3:** the docs say the hashed `BASE` detects an edit that was not re-hashed (an unkeyed integrity check). No claim of tamper-evidence or refusal of hand edits in general.
* **P4:** after collecting units, `proc_macro2::extra::invalidate_current_thread_spans()` is called, so the caller's thread keeps no span map.
* **P5:** GAP-13 closes in C6 by removing `serde_json`, not in C3. GAP-12's "reading does not gate the version" check is carried over to the RDF reader test.
* **P6:** the check is a committed test pinning the N-Triples rendering of the example reports, not a scratch parser run.
* **P7:** the design's `blake3 =1.8.7` is superseded by C4's caret floor. The Usage assessment joins the re-acceptance list. GAP-16's change-record item uses the migration capability's `max_file_bytes` bound.

## Rev 5: pass 4 advisories adopted
* Unit IRI key is symbol, file, line and start byte, with `census:startByte` a unit property (option a): in every unit shape, the page table, `tests/page.rs` and the typed rows. The one-line cfg fixture round-trips through `from_turtle`.
* The pinned N-Triples are the expansion by the independent parser (purrdf-rdf, run in a scratch crate on the final bytes; the run is recorded in validation.md); the test's doc comment says so.
* A `units.rs` test loads one tree twice on one thread, proving the span map is cleared.
* Fuzz targets are named in kebab case: `census-normalize` (C4) and `census-report-read` (C6), with files `census_normalize.rs` and `census_report_read.rs`.
* C5 adds a Constraint sentence: Census::load clears the calling thread proc_macro2 span table, invalidating proc_macro2 spans the caller holds on that thread (C2 review note).
