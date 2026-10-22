<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RDF results, sole coverage policy, and repository adoption

Read-only queue preparation, 2026-10-20. No builds, product edits, commits, or pushes were performed for this assessment. This file is ignored by `/.stage/`. Parent currently owns census integration and qualification; implementation must wait for that lane to finish.

## Delivery decision

Group **40 + 16 + 37 in one landing unit**, with three separately checked acceptance mappings. RDF results (40) can be implemented before the policy flip, but 16 cannot land honestly with the current repository adoption unpaid. Its sole/default policy also applies to the repository's `self_workplan` gate. Adoption (37) must therefore be ready in the same tested tree before landing the flip. A debt baseline alone is not zero-debt adoption. No hidden lightweight constructor, test-only opt-out, broad exclusion, suppressed finding, or automatic baseline is an acceptable bridge.

Parent scope clarification: 40 remains open until its actual census debt runtime acceptance (owned by 4) and census CLI/protocol consumers (6/12) satisfy the shared vocabulary requirements too. Workplan plus current census baseline/index foundation alone cannot close 40. Preserve these dependency/acceptance owners explicitly in the delivery queue. The existing census profile is internally complete; migrate its report profile explicitly when this delivery changes vocabulary/serialization, without changing normal-form hash bytes or NF merely for presentation.

Retain separate ownership in the workplan records: result serialization; sole policy and fixture enforcement; RAF documentation/code adoption. The final gate must inspect the actual combined tree. Do not close 37 on fixture proof or close 16 merely because old opt-in reviews passed. If a smaller independently complete 40 foundation is useful, it may land first with all its own command/format acceptance satisfied; the 16+37 default/adoption pair remains inseparable. Do not manufacture a partial 40 milestone that omits commands or vocabulary requirements.

## Authority and checkout

- `stagectl path --issue 16` resolves the existing checkout `.worktrees/16-raf-workplan-strict-mode-force-coverage`, stage `.stage/raf-workplan-strict-mode-force-coverage`. Reuse it; do not replace the donor or create another implementation lane.
- Fresh `stagectl issue 40 --json`, `issue 16 --json`, and `issue 37 --json` were compared with main `.stage/queues/portfolio/issues.json`. The newer instructions override the original opt-in/JSON acceptance.
- [16 default/RDF direction](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/16#issuecomment-6303767380): strict is the sole default; remove the strictness flag/key and lightweight mode; retain shrinking debt; new reports are RDF from the start.
- [16 adoption split](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/16#issuecomment-6286301260) and [current queue disposition](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/16#issuecomment-6305989004): preserve 37's distinct adoption acceptance; T1–T7 are reusable implementation, not proof of revised acceptance; dirty T8 is unfinished.
- [37 current direction](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/37#issuecomment-6303772305): RAF must satisfy the single mode, with zero first-party debt as the target; a large baseline does not complete adoption. Old issue counts are not current measurements.
- Read governing AGENTS, CLAUDE, `.goals`, debt-ratchet and OKF-profile architecture. `.baseline` is absent. Recall used explicit repository bank `repo-rust-agentic-foundations-d1e65424d255`; no recalled proposal supplies authority to waive current requirements.
- Graphify was consulted before cross-source navigation; main has no graph snapshot. Confirmed architecture directly against source; did not rebuild it. The landed source-root discovery and forthcoming census foundation must be merged normally at a clean checkpoint, preserving both contracts.

## Donor and review reuse

Observed HEAD `dc1115b`; old main integration `1145af3`. Product implementation is mostly committed. Dirty T8 is primarily docs, capability states/catalogue, assessment/evidence changes and two test edits, not a completed new-policy delivery.

| Existing work | Reuse and remaining obligation |
|---|---|
| T1–T3 | Reuse strict model/rules, claim and bound-public-item implementation. Their rechecks passed their historical scope. Replace opt-in assumptions and JSON expectations; recheck revised closed schemas and default callers. |
| T4 | Reuse package/target discovery, line accounting, ceilings and exceptions, including reviewed fixes through `973d7ab`/`df26e24`. Revalidate with landed source-root discovery. |
| T5 | `T5-review.md` is BLOCKED; no T5-recheck artifact exists. Commit `39717b1` and current `raf-source/src/reach.rs` contain const/raw-pointer, edition-aware `gen`, Self/method reachability and budget/count fixes. Source presence is not an independent review pass. Recheck the actual R1–R5 cases, counted work and valid neighbors. Foreign IDs mean out-of-range IDs, not a promise of cross-workspace provenance. |
| T6 | `T6-recheck.md` passed at `fec6c21`; retain CLI/report coverage, nested workspace, ceilings/exception validation. JSON and flags are superseded. |
| T7 | `T7-recheck.md` passed at `dc1115b`; retain all ten debt kinds, re-export and completion-after-record-removal behavior. Remove the requirement to opt into debt enforcement; retain identity/shrink-only validation. |
| Dirty T8 | Preserve and inspect each doc/test/assessment delta. It marks nine capabilities delivered and removes the old change from the docs inventory before the actual record has completed. `work/changes/workplan-strict.md` still exists with old opt-in outcomes. Do not take those outcomes, reduced matrix tests, or changed inventory as current completion evidence. |

The dirty line tests add impl-orphan/named-type coverage and distinguish coverage exclusions from explicit line exceptions; preserve those substantive cases. The touched fixture adds `impl Eq for Frame {}`. Re-recorded evidence must retain provenance and SPDX. Historical mutation runs in validation are historical only; the current ban forbids rerunning them. Historical toolchain failures are neither current results nor permanent waivers.

## Concrete RDF architecture proposal

Use a small shared **pure Rust RDF kernel with two actual consumers**, rather than make workplan depend on census or generalize census's private graph into a framework. It supplies owned ground terms/triples, lexical writing via `purrdf-lex`, deterministic ordering, and shared vocabulary/profile constants. Keep workplan result mapping and census closed shape/semantic validation in their own crates. No generic schema engine, serde value tree, dynamic object model, blank-node facility, or second lexical implementation is needed.

Use one documented, IRI-versioned namespace, proposed `https://blackcatinformatics.ca/raf/report/1#`, for the shared result vocabulary. Census normal-form identity IRIs retain their independently versioned `.../census/nf/{normalForm}/...` meaning. The current census `.../census#` namespace is unversioned: `reportForm` alone does not meet 40's IRI-versioned vocabulary requirement. Migrate baseline/index and the designed debt vocabulary explicitly; update the closed reader, profiles, exact fixtures and independent pins. Bump the census report profile for a changed delivered byte/IRI contract; do not bump NORMAL_FORM merely for serialization. The first-merge normal-form decision remains intact.

Provide canonical N-Triples 1.2 and canonical Turtle 1.2 over **the identical ground graph**. N-Triples uses absolute IRIs and no directives; Turtle may use fixed prefixes/BASE. Presentation cannot participate in identity. Specify a single framed semantic preimage (version tag plus sorted ground triples with an explicit reserved report-self token before report-local IRI expansion), hash it with full BLAKE3, then mint absolute report/entity IRIs. Document framing and reserved-token refusal; no circular hash, random ID, timestamp, platform path spelling, or format-dependent BASE. Stable natural document/symbol/claim/evidence identifiers remain stable; occurrences whose identity depends on context include that context. Compare two formats after parsing as graphs, not by textual substitution.

Census currently hashes canonical Turtle body bytes excluding its BASE line; that law cannot be copied unchanged into an N-Triples emitter while claiming format-independent identity. Treat the new shared preimage and report profile as an explicit migration, with refusal fixtures and exact byte goldens. Native reports remain closed and canonical. There is no reason to add a permissive legacy fallback; any supported foreign JSON import must be explicitly named, one-way and outside native readers. Existing C6 native baseline/index paths are already RDF; do not resurrect JSON to provide compatibility.

Use `purrdf_lex::term_syntax::{write_iri, write_literal}` and the RDF 1.2 triple-term delimiters; use its percent/IRI helpers where their contract fits. Do not copy escaping or use `write_blank`. The existing census percent encoder and minting laws need to move/share only where both consumers actually need them. Preserve injective separators and start-byte disambiguation. Keep bounded triple-term nesting, validate all terms, avoid per-comparison owned-string allocation or graph rebuilds, and state map/sort costs accurately rather than promising a linear whole reader.

### Result graph and provenance

Root `Report` has profile/version, command, result status and ordered result membership. Named typed result nodes cover `Finding`, `Obligation`, `Binding`, `ItemState`, `Evidence`, `Assessment`, `CoverageClaim`, `CoverageLine`, `EntityHistory`, `DebtEntry`, and command outcomes. Required scalar types are IRI, string, boolean, integer and exact typed numeric values; absent is absent, never JSON null encoded as a string. Give each command's variants a closed shape; reject invalid required/mutually exclusive fields before rendering.

Use named typed statement/reifier nodes and `rdf:reifies` with RDF 1.2 triple terms to state which binding, digest, evidence or actual run supports a claim. For example, a named binding assertion reifies `<<( capability binds symbol )>>`, then links to its definition/source digest; a verification assertion links to a real run and its result. A marker stating a coverage claim is not itself proof of a successful verification run. Preserve both assertion and supporting provenance rather than flattening either into an opaque message. No RDF 1.1 reification, blank nodes or quads.

Document every emitted class, property, datatype and enum IRI. Test both directions using independent emitted fixtures and the documented registry, including refusal/outcome variants; generating both from one table alone would not prove all mapper branches are covered.

## Command and JSON elimination map

Replace both current serialization routes: `render::Record.fields` contains `serde_json::Value`, while `Rendered` holds a separate pre-rendered JSON string. Library `coverage/render.rs` and `history/render.rs` also hand-write JSON even though raf-workplan has no serde_json dependency. All must converge on the typed result graph. Keep human text rendering and its diagnostic escaping.

| Terminal command family | Preserve complete result information |
|---|---|
| check; status | Findings/kinds/locations; capability states; entries and open changes. |
| docs sync; docs preview | Rewritten paths; generated/preview document contents and refusal findings. |
| change new; impact; reconcile; complete | Template/capability/affects/guidance; bindings/resolution/dependants; obligations; item outcomes; completion conditions/repeated/message; removals and rewrites. |
| verify | Binding evidence, capability/list/path, proof, outcome/reason and diagnostics. |
| evidence record; claims; for | ID/path/verdict/subject/command/revision/by/covers; exact values and limits; input kind/name/digest; failure code/names/reason; claim section/line/statement and supporting verdict. |
| assess context; verdict; list | Document/section/symbol definitions and unresolved names; inputs/evidence; recorded verdict/by/replacement; currency and moved inputs. |
| history | Commit/date/subject, symbol/document sources and paths; removed-binding lookup including not-found/no-commit/no-crate and last resolved states. |
| coverage code; tests; claims; lines; reverse alias | Covered/orphan/excluded/error entries, source/target/root/re-export context; claim key/capability/section/line/tests; crate/module/item line ranges, roles, counts, percentage/ceilings, exceptions and reasons. |
| debt; debt --write | Current debt, stable identities, baseline changes/refusals and write outcome, preserving shrinking ratchet rules. |

Accepted machine formats are `--format ntriples` and `--format turtle` (document exact spellings); default text remains. Every terminal command and relevant alias must reject `--format json` at argument parsing with exit 2 before mutation or execution. Preserve exit 0 success, exit 1 findings/refusals, exit 2 usage/execution errors; do not turn a reportable refusal into a successful empty graph. Define quiet behavior explicitly: text retains findings; machine output remains a valid report with accurately filtered membership, never malformed or a fake success. Preserve existing stderr semantics for parse/early execution errors.

Remove cargo-workplan serde_json from normal AND dev dependencies, rewrite JSON test support/fixtures, remove manual JSON helpers and check docs/examples. Add the requested repository gate over every first-party Cargo.toml, including fuzz/auxiliary manifests, with a negative fixture and valid neighbor; then prove `cargo tree -p cargo-workplan` has no serde_json. No feature-gated legacy JSON path. Coordinate census integration before judging the whole-repository gate. Markdown/OKF remains the source-document and workplan-debt policy storage: 40 changes machine results, not every document into RDF. Keep 41's separate document-projection scope explicit.

## Sole policy revision and adoption mapping

1. Remove `Args.strict`, Gate strict/strictness setters/floor, Workplan effective-strictness plumbing, the public Strictness choice where obsolete, and the accepted coverage `strictness` key. Old flags/keys fail closed with actionable diagnostics; no ignored compatibility knob. Run all production open/check/reconcile/complete/coverage/debt callers through the same policy. Remove `debt::check_default`/write opt-in refusal while retaining real debt validation.
2. Retain and enforce every original substantive requirement: runnable example or doctest and verification matrix per delivered capability; Contract/Constraints markers linked to bound tests; all bound-package public items explained; completion verifies every binding, tense and touched-public-item `none` restrictions; minimum sections/Usage/Examples/limitations; dropped/moved reasons; closed finding kinds with both-direction tests.
3. Preserve reverse source coverage: per-crate/module/item ranges, bound-item transitive reachability, shipping/test classification, deterministic path/target handling, ceilings/percentages, explicit reasoned exceptions and strict exclusions distinct from line exceptions. Preserve current source budgets; no timing-based complexity claims. Verify all original T5 repairs rather than silently accepting unresolved review status.
4. At the settled integrated tree, produce real adoption inventory by capability, finding kind, package, public binding and line ownership. Map each finding to an actual 37 code/doc repair. Do not treat the old issue counts or dirty T8's nine state changes as measurements. Cover all delivered first-party crates, including newly delivered census portions, without falsely declaring designed census capabilities delivered.
5. Repair RAF examples, matrices, claim markers, public explanations, source/verification bindings, Usage/Examples and precise limitations. Resolve or explicitly justify narrow source exceptions under the documented rule. Zero debt means no retained debt entries; a shrinking baseline can enforce interim development but cannot satisfy 37's terminal target. If counted inventory changes the coherent scope materially, update the same delivery plan visibly, not an unowned deferral.
6. Reconcile architecture, AGENTS, README, CLI formats/exits, capability states/catalogue and designed downstream contracts against actual delivered behavior. Census debt `.json` wording in the old debt-ratchet ADR must reconcile with the landed RDF foundation. Refresh assessments after source/docs settle; preserve SPDX through lifecycle rewrites. Complete the actual fresh change records with the lifecycle gate, deleting records only after successful completion. Keep inventory and real records in agreement.

## Activation and qualification sequence

After parent lands census and releases the implementation lane: preserve donor diff/evidence, synchronize actual origin/main normally, resolve conflicts substantively, and compile all affected real callers early. Review the reused T5 fixes and dirty T8 before accepting their claims. Establish the shared RDF profile/vocabulary and exact provenance shapes, then migrate every command and census profile with independent parser fixtures. Implement sole-policy removal and actual 37 repairs in the same checkout, not a parallel hidden adoption lane.

Focused validation: production API/CLI default tests; old flag/key rejection with valid neighbors; every command JSON rejection and both RDF formats; deterministic repeated serialization; independent N-Triples 1.2 graph replay and Turtle equivalence; named triple-term provenance/no blank nodes; vocabulary two-way coverage; dependency manifest gate/tree; all strict rules and debt shrink/completion refusal neighbors; counted source-work/budget fixtures. Negative cases must actually be run, not inferred from type design.

After final docs, accepted assessments and successful lifecycle completion, run required root gates and one settled full `make check` with exact exit/log evidence, plus ordinary mandatory commit hooks. No historical gate substitutes for revised acceptance. Parent handles independent substantive review, normal commits and ghprsq landing/cleanup. Keep 40, 16 and 37 acceptance lines separate in final validation even when one landing unit satisfies them.
