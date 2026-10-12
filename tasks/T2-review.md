# T2 review (independent)

## Scope
LICENSE-MIT, LICENSE-APACHE, LICENSE-MULAN, LICENSES/{MIT,Apache-2.0,MulanPSL-2.0,CC-BY-4.0}.txt, REUSE.toml, LICENSING.md, docs/LICENSING.zh-Hans.md, docs/LICENSING-BACKTRANSLATION.md. Read-only review.

## Evidence
- sha256sum LICENSE-MULAN, LICENSES/MulanPSL-2.0.txt -> eb7a1d71...bf1c (match). LICENSES/CC-BY-4.0.txt -> d557539d...3744 (match). rc 0.
- cmp vs purrdf: LICENSE-MIT/APACHE/MULAN, LICENSES/MulanPSL-2.0.txt, CC-BY-4.0.txt identical (rc 0). purrdf has no LICENSES/MIT.txt or Apache-2.0.txt; instead the new copies are sha-identical to LICENSE-MIT / LICENSE-APACHE (958c0e61..., cfc7749b...). Satisfies R11b.
- LICENSING.md vs sed-substituted purrdf: sections present (Open-source terms, Documentation, About MulanPSL-2.0, Third-party material, Proprietary/commercial, Trademarks, Contributions, Copyright notice). Vendored corpora table, Unicode, bindings, GTS, release-artifact sections absent. grep for purrdf|unicode|vendored|binding|gts|npm|wasm in LICENSING.md and zh doc: no hits.
- Doc rule: LICENSING.md "Documentation" section (every Markdown is CC-BY-4.0; code/config/templates/brand assets three-licence) and REUSE.toml header comment agree; REUSE.toml annotates INITIAL_PLANNING_PLAN.md as CC-BY-4.0 consistently.
- zh diff command: rc 1 with only deletions (the purrdf third-party/Unicode/artifact paragraphs) plus line 1 header, where the copyright holder gains the registered mark (Informatics® Inc.). Not a content change.
- Remaining zh claims (three-licence choice, commercial not a fourth option, CC-BY retained, Mulan Art. 6/2/3/4, digest-pinned) do not contradict LICENSING.md.
- sha256sum docs/LICENSING.zh-Hans.md = 01eb7226...7aa0 and wc -c = 1692; backtranslation record states the same hash and 1,692 bytes. Comparison bullets accurate against LICENSING.md (the digest line is covered by LICENSING.md "verified by make licenses").
- `uvx reuse@6.2.0 lint`: compliant, 12/12 files, rc 0.
- grep for this issue|the plan|Task N|#NNN in the files: none. Brand casing RAF consistently (the only lowercase is SPDX-PackageName "raf", a package id).
- make licenses: plan.md Task 3 lists the `licenses` target and adv. 12 specifies Mulan digest plus zh fingerprint comparison against the backtranslation record, so the forward reference is covered.

## Findings
REQUIRED: none.
ADVISORY:
1. Backtranslation timestamps (2026-10-12T05:13Z) are consistent with the current UTC clock; if the zh file is edited, hash and timestamps must be refreshed (Task 3's check will enforce the hash).
2. zh doc says "Documentation bearing the CC-BY-4.0 identifier retains that licence", slightly weaker than LICENSING.md's "every Markdown document"; not contradictory, left as is (changing it would invalidate the backtranslation).
3. LICENSING.md links CONTRIBUTING.md, which arrives in Task 4; link is dangling until then.

VERDICT: PASS
