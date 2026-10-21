<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Full identity and selection evidence

Implementation base is C6 commit 29df7f6. Source is exclusive to the existing census checkout. NORMAL_FORM remains 1 under the first-merge decision.

* Body and shim fingerprints retain every BLAKE3 byte, strict canonical Display/parser uses 64 lowercase hex, and abbreviated() is separate human text. Full suffix tests prove grouping/equality do not use abbreviation.
* Print holds Option<Fingerprint> directly. Removed Hashed, duplicate full digest and obsolete truncated-key collision guard/error. Actual full-digest collision resistance is the BLAKE3 assumption.
* Untrusted public parser errors escape controls; refusal and accepted neighbors are tested.
* Literal-value introspection is on demand, sharing the production passes. Source analysis shares the Self:: predicate, parameter facts and target deduplication. No production tuning option was added.
* BLAKE3 requests default-features=false, pure. Dependency build.rs was inspected; exact 1.8.7 no_std hash-only wasm32 probe compiled exit 0 (raw/identity-hash-wasm-probe.log, scratch/portable-hash source and lock). This does not establish full raf-source/census wasm support; that split/gate remains tracked separately by the portfolio.

## Settled-source measurement

raw/selection-counts.ttl and .log are the final run, exit 0, after the single-representation code and count-tool tests settled. Exact Rust delta from 29df7f6: raw/selection-measured-source.patch; the new example not present in that base is copied verbatim to scratch/selection-measured.rs. No measured crate Rust changed after that measurement; the root documentation-test inventory adds the new Explanation assessment. Current count rationale is docs/census-selection.md, with exact selected RDF rows at docs/data/census-selection.ttl. The raw run contains all 432 policies for each corpus, bins, and concrete inspection groups.

Inputs: fixtures; the five RAF crate roots via the checkout Tree; cached blake3 1.8.7, purrdf-lex 3.0.1, serde 1.0.229, syn 3.0.8, toml 1.1.8+spec-1.1.0, quote 1.0.47 under /home/paudley/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/. Exact paths are retained in RDF Input records; Cargo.lock records registry checksums. No source copies of those caches were made.

Earlier failed Tree-root choice is preserved in raw/selection-preliminary.ttl/log with five explicit workspace-inheritance refusals. raw/selection-before-final.ttl/log is preliminary pre-cleanup evidence and must not be cited as current counts. Correcting the Tree root kept the same source packages and paths; no sample substitution occurred. A missing-directory refusal run is raw/selection-refusal.ttl/log; expected exit 1, beside the successful eleven-source-root final run.

Manual inspection covers toml ValueSerializer and KeySerializer overload families (133 of 212 removed floor16→20 pairs), syn iterator size hints and generated Hash traversals, workplan list-parser string fixtures, syn debug field labels, trivial Ok(()) shim constructors, and Self::ALL from_name methods whose shim exclusion disappears under an overly short shim upper bound. Full member locations and normalized forms are retained in raw RDF Inspection nodes. These are observations on syntax, not an independent real-source precision label set. The tiny serde written-body count and unexpanded macros are explicit limitations.

## Qualification

* raw/identity-clippy-current.log: raf-census all targets clippy, exit 0.
* raw/identity-census-current.log: raf-census all targets tests, exit 0; 131 tests (20 library, 26 forms, 4 golden, 23 groups, 7 page, 25 reports, 25 units, 1 selection example).
* raw/identity-fuzz-clippy.log: fuzz manifest all-target clippy, exit 0.
* raw/identity-oracle.nt/log: independent pinned purrdf parser expansion of current baseline/index/awkward bytes, exit 0; 41/89/11 triples. Sorted expansions update the pinned .nt fixtures. Existing retained oracle source/lock is under scratch/rdfprobe/oracle.
* raw/identity-counts-oracle.nt/log: independent parser accepts the selected count RDF, exit 0; 3588 triples.
* Full workspace make check has not run on this change. Parent coordinates it after normal commit and integration of actual origin/main (source root discovery).

Lifecycle and final focused root gate results are appended when terminal. No complete-issue or merge claim is made by these focused checks.

## Substantive generic-callee repair

Parent review found actual generic-path errors. raw/identity-self-callee-both-before.log reproduces both directions plus higher-rank function types. Shared balanced generic-list scanning now skips function-type groups and ignores return-arrow closing punctuation. Direct cases and loaded trait-default neighbors pass (identity-self-callee-after.log, identity-self-units-after.log). The same included source fixture compiles independently with warnings denied (identity-self-generics-rustc.log, exit0). Its compiler output is disposable.

identity-self-census.log: all133 raf-census tests pass. identity-self-clippy.log: all-target clippy passes. Prior131-test evidence predates this repair. selection-pre-self-callee.ttl/log/source.patch preserves the prior snapshot. Current selection-counts.ttl/log is the post-fix replay:417 census bodies,50664 tokens,9009 selected source units. Group/pair counts are unchanged. Only432 census Count rows change; other4752 rows are identical. Current selection-measured-source.patch includes measured crate source and the new fixture/helper/tests. Root documentation inventory is outside this corpus.

The first completion refused a missing Explanation assessment; the next was deliberately stopped (143) before source repair. Neither is a pass. The current record includes the repair and seven assessment inputs for the final gate rerun.

## Terminal handoff

Actual final lifecycle completion: exit 0, record removed (identity-self-complete.log); retained complete record has five implemented outcomes. Root docs_contract 28, repo_gates 68, self_workplan 1: exit 0 (identity-root-gates-current.log). Earlier root shape failure is superseded by the explicit Explanation inventory counts: 63 assessments, 65 total records; uniqueness/other prefix counts remain enforced. Standalone terminal deny and reuse both exit 0 (identity-deny-terminal.log, identity-reuse-terminal.log). All seven census assessments are accepted/current. Parent independent review PASS is tasks/queue-c7-review.md. Source and metadata ownership return to parent for normal commit/push and main integration. No full workspace gate, commit or push by this lane. Full parent WASM acceptance stays open through #46.

Currentness-log clarification: identity-workplan-check-final.log is a preserved pre-completion delivered/open-record failure, not current pass evidence. Successful lifecycle re-evaluation and the terminal root self_workplan pass establish post-removal currentness.
