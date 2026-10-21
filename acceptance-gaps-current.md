<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Current census acceptance and remaining scope

Source of current direction: [maintainer comment 6303763767](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/3#issuecomment-6303763767), 2026-10-20 23:05:11Z.

Exact additional requirements:

> Specify the census core as pure functions over a `Tree` (normalize, fingerprint, group, report) with no filesystem or process access, keep the `cargo metadata` reference scan in the host layer, and confirm `blake3` is used in its portable configuration so the core builds for wasm32. The wasm gate itself is tracked in #46.

> Use at least 128 bits (or the full digest) as the identity and print a short prefix for people. Justify thresholds and tiers by measurement on the corpus (counted work, no timing) rather than by inheritance. Changing what a fingerprint hashes bumps `normal_form`.

## Ownership and superseded directions

- Main `.stage/goals-survey.md` row 14 maps the census core boundary/portable hash to #3, and reference scanning to #5. Row 24 maps fingerprint width and corpus justification directly to #3. Its existing-issue table repeats that ownership. There is no separate owning delivery for width or threshold justification.
- #46 owns the shared wasm split/gates and explicitly includes the census pure-Tree boundary and portable hash feature set. Actual wasm compilation is presently blocked by raf-source's 32-bit level representation, as recorded by the RDF design. This is an explicit tracked prerequisite, not a claim that wasm acceptance passed here.
- The later [remediation comment 6304175085](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/3#issuecomment-6304175085) says goals elsewhere are handled from the survey. Filing those comments did not implement or waive #3's requirements.
- The original body's first-eight-byte identity is superseded by the later at-least-128-bit direction. The original JSON reports are superseded by the explicit user RDF 1.2 decision and C6 amendments. The user-confirmed first-merge rule permits defining and regenerating normal form 1 before its first merge; it does not waive the identity requirement. The old C2 reparsing-on-a-large-stack proposal is superseded by Rev 2's one-parse source facts.

## Current requirement mapping

| Current requirement | Owning implementation / evidence | State and missing acceptance |
|---|---|---|
| Token-stream bodies, macro arguments, structural/value/literal/shim forms | normalize modules; fixture registry/goldens; original body B1-B5 | Built before C6; C1 corrections committed. Full final gate remains pending. |
| Keep item names, rename every bound form, first appearance and field/control-head rules | C1, gap 01-04; groups/units/pairs/page tests | Historical focused qualification exists; final changed tree still requires qualification. |
| Fingerprint identity at least 128 bits, short human form | C7: Body/Shim hold [u8;32]; strict64 canonical display/parser, separate human abbreviation, full-suffix tests | Implemented; independent recheck and combined-tree full qualification remain. Obsolete Hashed/truncation guard removed. |
| Corpus-derived thresholds and tiers | C7 selection example; docs/census-selection.md and docs/data/census-selection.ttl; 261 fixtures and eleven exact source corpora, 432 policies each | Implemented. Post-fix source counts: 9009 selected, 155 body groups + 49 shim groups, 393 pairs. Inspected disagreement groups support 20/30/6–29 with explicit macro/label limits. |
| Units with symbol/package/file/line/shipping/print/constants/tables/strings | units collector, C2/C3; 25 units tests before C6; startByte added by C6 | Built; startByte/cfg same-line report neighbor is in current tests. Final broad coverage pending. |
| Deterministic multi-member grouping, hex-table detection, shared large constants | group.rs, normalization scans, groups tests | Built; no hash/group behavior redesign in C6. Final qualification pending. |
| Safe one-parse parameter/initializer facts and bounded stack | C2 source facts and small-stack tests; C3 hard index failures | Committed and reviewed; source parse itself belongs to raf-source. |
| File target kinds and hard invalid-id/empty-reach errors | C3 fe26b39, independent review/recheck, focused units/root gates | Complete behavior; current final gate still pending. |
| Dependency/security/fuzz surface and downstream obligations | C4/C5 b1ba90e, normalize fuzz + security/docs | Committed, focused evidence recorded. C6 expands reader fuzz/security. |
| RDF 1.2 deterministic baseline/index, closed canonical reader, version-free normalForm and strict reportForm | C6 committed/pushed 29df7f6 with independent recheck and normal hook | C7 regenerates full-identity bytes/pins. No whole-issue completion claim. |
| RDF identity, typed named match reifiers/triple terms, startByte uniqueness, awkward escaping | C6; baseline/index/awkward Turtle + independent N-Triples pins, cfg fixture | Focused tests and independent parser pass; final review/qualification pending. |
| No census serde/serde_json; shared purrdf lexical laws, audited license exception | C6 manifests/locks; deny/reuse passing | Implemented; report-reader fuzz run passed 5000 runs before final review refinements. Mandatory final gate remains required. |
| Normative architecture and reconciled capability; downstream designed docs kept accurate | normal-form architecture/capability, designed CLI/migration/debt/ledger docs | C6 docs-sync and assessment refresh performed; root gates and review still being collected. |
| Golden coverage, distinct called items, collapsed binder/noise neighbors, byte-identical runs | goldens/pairs/groups/units/reports/page tests | Existing substantive checks retained; current focused report tests green. Final full make check + Linux hosted jobs still required. |
| Pure Tree census core and host-only reference scan | C7 explicit Tree contract; BLAKE3 no-default-features + pure; inspected build.rs and exact1.8.7 no_std hash-only wasm32 probe | Hash-only probe passes; full source/census wasm gate remains owned by #46 and is not claimed here. |

## Current qualification and next owned steps

C6 is committed as29df7f6. C7 implements the remaining identity, corpus and boundary requirements above. The fresh census-identity-and-selection workplan record covers all seven sections and five concrete outcomes. Its first actual completion refusal identified the missing Explanation assessment; the next run was interrupted (143) for substantive scanner repair and is not pass evidence. The completed record source is retained under stage.

The generic-callee repair distinguishes associated types from actual Self calls with lifetime and function-pointer arguments. Direct query cases, loaded trait-default neighbors and a standalone warnings-denied Rust compilation verify it. The post-fix measurement preserves prior snapshots and source deltas. All non-census corpus rows and all group/pair counts are unchanged.

1. Complete the actual lifecycle gate and focused root qualification on settled C7; retain exact results in validation and tasks/identity-selection-evidence.md.
2. Parent independent recheck and normal commit/push; source ownership transfers only on explicit handoff.
3. At the clean checkpoint integrate origin/main containing source-root delivery34dac35 (PR51); preserve its root/manifest contract and census C2 one-parse source facts and refresh source-index metadata against both.
4. Run mandatory full make check and hosted qualification on the combined tree, then land through ghprsq. This implementation lane has not run the combined full gate.

## Terminal C7 handoff

Actual completion gate passed and removed the fresh record; retained source has five implemented outcomes. Seven assessments are accepted/current. Root focused gates pass 28/68/1 with terminal exit 0; deny/reuse both exit 0. See the current header of validation.md. Independent parent review PASS is tasks/queue-c7-review.md. Source ownership returns to parent. Full #3 WASM acceptance remains open through #46; this foundation delivery does not close #3. Main integration and the mandatory combined full gate remain required.
