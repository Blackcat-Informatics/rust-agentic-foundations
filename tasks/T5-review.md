# T5 review (Branding and README)

## Scope
Uncommitted: docs/BRAND.md, docs/raf-logo.svg, docs/social-preview.svg, docs/social-preview.png, README.md, REUSE.toml (PNG annotation). Contract: plan Task 5, A13, A17; R21-R24, R43. No source files modified.

## Evidence (all rc 0 unless stated)
- R22 cat-head-core awk hash = 4ec35cf6551175dbcb7bed900ccf626aaf6623d73e29b39dc83c8468eb6a97fe (match).
- service-toolbox count 1; service-triple count 0.
- Accent isolation script prints [] rc 0. Accent hex appears only in toolbox-* / wrench-body attributes (lines 105-114); `<style>` and class= exist but carry no accent.
- xmllint --noout both SVGs rc 0. toolbox mentions 15, wrench 6.
- PNG: 1280 x 640 RGB. `rsvg-convert -w 1280 -h 640` re-render, cmp rc 0 (identical).
- social-preview.svg: RAF x4, purrdf/RDF 1.2 count 0.
- Logo rendered to PNG and viewed: family black cat biting the handle of a cream/red toolbox with blue hinges, yellow latch, green wrench across the front. Correct. Social-preview PNG viewed: correct.
- `make issue-refs` rc 0. `uvx reuse@6.2.0 lint` rc 0 (32/32 compliant).
- purrdf greps: `git grep -ni purrdf -- . ':!docs/BRAND.md'` no output; README/SVG/REUSE no match. R20 word grep over README/BRAND no output.
- BRAND.md: "RAF (Rust Agentic Foundations)" count 1; sections Name, Positioning, Tagline, Repository description, Family system, Colour tokens, Logo assets; service-toolbox 2; rebuild command 1; all 7 colour tokens present.
- REUSE.toml PNG annotation: precedence aggregate, copyright Blackcat, MIT OR Apache-2.0 OR MulanPSL-2.0. Consistent with A13 (brand assets = code licence) and REUSE.toml header comment.
- README: 6 crates each "designed - not yet published"; usage under "Planned usage" stated "planned and is not available yet"; no published/implemented claims; raf-okf, raf-source, raf-census, cargo-census, raf-workplan, cargo-workplan present; Gate::workspace snippets for both; `dev-dependencies` present; logo linked; "RAF" prose casing, lowercase only in machine ids (R43).

## Link targets
Capability docs (to be created by T6/T7; confirm in final review): docs/capabilities/{okf-profile, source-index, census-normal-form, census-cli, workplan-document-model, workplan-cli}.md. All six are within the planned slug set; none outside it.
Other links, existing now: AGENTS.md, CONTRIBUTING.md, LICENSING.md, docs/BRAND.md. `make hooks` target exists.

## Findings
REQUIRED: none.
ADVISORY:
1. README intro uses present tense ("A census finds ... ratchets ... reconciles ... Both are built to ... fail hard") before the "designed, not yet published" notice in the Crates section. Not a shipped-claim, but a leading "Planned"/"designed" qualifier in the opening paragraph would remove any ambiguity (A17).
2. docs/social-preview.png is byte-reproducible only on this machine (font fallback, K6); correctly not CI-gated.

VERDICT: PASS
