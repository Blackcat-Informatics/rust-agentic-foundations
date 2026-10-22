# Issue #40: workplan and census machine output: replace JSON with RDF 1.2

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-census, crate:cargo-census, crate:raf-workplan, crate:cargo-workplan

## Body

## Context
Every machine-readable surface of `cargo workplan` is JSON: `check`, `status`, `sync`, `preview`, `impact`, `reconcile`, `complete`, `verify`, `new`, `evidence`, `assess`, `history` and `coverage` each offer `--format json`, each built by hand as a `serde_json::Value` (`crates/cargo-workplan/src/render.rs:26,107,208-230`, `change.rs:21`, `assess.rs:18`, `command.rs:11`, `evidence.rs:26`; documented in `docs/capabilities/workplan-cli.md:54-93,163-165,181`, `workplan-entity-history.md:181`, `workplan-two-way-coverage.md:286`). The strict-mode work in the open PR for #16 adds three more (`coverage claims`, `coverage lines`/`--reverse`, `debt --write`). The census side (`raf-census`, `census-debt.json`, `baseline --json`, `index`) has the same shape; census reports are already being replaced by RDF 1.2 reports in PR #39 by decision.

Goal: RDF 1.2 only. "If you find yourself using JSON you have failed." RDF 1.2 and triple terms are first-class, not a short cut.

## Scope
- [ ] Define one RDF 1.2 vocabulary (IRI-versioned) for findings, obligations, bindings, item states, evidence, assessments, coverage claims and entity history, shared with the census report vocabulary from PR #39.
- [ ] Use RDF 1.2 triple terms for provenance (which binding, which digest, which run backs a claim); no blank-node reification and no RDF 1.1 or quad shortcuts.
- [ ] Emit canonical N-Triples 1.2 (byte-stable order) and Turtle 1.2 as the only machine formats of every `cargo workplan` result command; the text format stays for people.
- [ ] Remove `--format json` and the `serde_json` dependency from `cargo-workplan`; the three strict-mode reports (claims, lines, debt) are defined in the vocabulary from the start.
- [ ] Update the CLI capability documents and the exit-code and format sections to the RDF forms.
- [ ] Extend the RDF output to census `baseline` and `index` and the debt record (`census-debt.json`) through the same vocabulary, with legacy JSON baselines accepted only as a one-way import of foreign data.

## Acceptance criteria
- [ ] `cargo tree -p cargo-workplan` lists no `serde_json`, and `cargo workplan <command> --format json` is rejected with exit code 2 for every result command.
- [ ] For each result command, a fixture run emits N-Triples 1.2 that a conforming RDF 1.2 parser round-trips to an identical graph, and two runs on the same input are byte-identical.
- [ ] A provenance statement about a binding is a triple term in the output, asserted by a test that parses the output and finds no blank-node reification pattern.
- [ ] The vocabulary document lists every emitted predicate and class, and a both-directions coverage test fails when an emitted term is undocumented or a documented term is never emitted.
- [ ] A repository grep gate fails on `serde_json` in first-party `Cargo.toml` files.

## Related
- PR #39 (census reports as RDF 1.2)
- #16 (strict mode adds new reports), #4 (debt record), #6 (CLI `--json`, `index`), #12 (CLI-to-target report protocol)
- #41 (OKF RDF projection), #50 (Python surface consumes these results)


## Comments (1)

### paudley — 2026-10-21T03:57:00Z

The delivery queue now groups #40 + #16 + #37 in one implementation/landing unit, with separate acceptance for each. The existing strict checkout and PR #38 remain the surviving implementation. Historical opt-in/JSON reviews are useful evidence for their tested scope; they do not qualify the revised sole-default/RDF contract.

The dependency is concrete: after strict becomes the only policy, RAF's existing self-workplan gate immediately judges the actual repository adoption findings. The default flip therefore cannot land with adoption unpaid, a hidden lightweight mode, broad exclusions, or a baseline presented as zero debt. We will inventory the integrated repository, repair the first-party documentation/bindings/claims/source coverage, and complete the actual lifecycle before the flip lands. #37 retains its distinct zero-debt target and shrinking ratchet acceptance.

Machine results will use the shared IRI-versioned RDF 1.2 vocabulary, canonical N-Triples/Turtle and named triple-term provenance across every workplan result command. JSON format rejection and dependency/manifest gates remain explicit. Current census report profiles will migrate explicitly to the common vocabulary/identity law; unchanged fingerprint hash bytes do not require a normal-form bump.

This grouping does not close any issue administratively. In particular, #40's remaining census debt-record behavior stays mapped to #4, and the census CLI/protocol in #6/#12 must use the same vocabulary. The #40 parent remains open until its complete census acceptance is proved. Likewise, #3 remains open through the shared full-core WASM acceptance in #46 even after its coherent foundation lands.

No third implementation lane has started. Census integration/qualification and resolver correctness are the two active lanes; the shared strict/RDF delivery is prepared for activation after census lands.


