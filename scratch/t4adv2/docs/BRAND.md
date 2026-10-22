<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAF Brand

## Name

The project name is **RAF**, short for *Rust Agentic Foundations*. Always write
it in capitals as `RAF` in prose and titles. Spell out
"Rust Agentic Foundations" on first use in a document.

All machine identifiers stay lowercase: the library crates are `raf-okf`,
`raf-source`, `raf-census` and `raf-workplan`, and the cargo subcommand crates
are `cargo-census` and `cargo-workplan`.

## Positioning

RAF is dependable dev-dependency tooling that keeps Rust repositories free of
duplicated helpers and keeps their documentation an accurate, current account of
the system. It is built to be driven by humans and coding agents alike. It should
read as quiet, strict infrastructure that a project adds once and then relies on,
rather than as an application or a framework.

## Tagline

Dependable gates that keep Rust repositories free of duplicated helpers and their
documentation true to the system.

## Repository description

RAF (Rust Agentic Foundations) — dev-dependency tooling for Rust repositories: a
duplicate-helper census with a debt ratchet, and a spec-first workplan that
reconciles capability documents with code, tests and examples, for humans and
coding agents alike.

## Family system

RAF shares the black-cat silhouette of the cat-head logo family. The shared
`cat-head-core` SVG group is reused verbatim from the family's origin
repositories (`purrdf`, `gmeow-ontology` and `gmeow-gts`). Keep the family
recognisable by never editing that group and swapping only the **service
object** held by the cat.

This repository's service object is `service-toolbox`: a **toolbox** whose
handle sits in the cat's mouth, with a **wrench** crossing its front. The
`mouth-bite` group is drawn above the service object so the cat bites the
toolbox handle.

The four accent colours (red `#ea4335`, blue `#4285f4`, yellow `#fbbc05`, green
`#34a853`) are isolated to the `service-toolbox` group, as fill and stroke
attributes, so they can be themed without touching the cat. No shared style rule
carries an accent.

## Colour tokens

- cat / ink: `#111214`
- paper (toolbox body, backgrounds): `#fffdf5`
- feature (eyes, whiskers): `#ffffff`
- accents: red `#ea4335`, blue `#4285f4`, yellow `#fbbc05`, green `#34a853`

## Logo assets

- `docs/raf-logo.svg` — the canonical RAF logo (cat, toolbox and wrench),
  including a soft white glow so the black silhouette stays legible on dark
  backgrounds.
- `docs/social-preview.svg` — the editable GitHub sharing-card source
  (1280×640).
- `docs/social-preview.png` — the rendered 1280×640 GitHub social preview.

Use the SVG for README, icon and card placements. The README references the logo
by relative path so branch previews render the branch's current logo.

Rebuild the PNG after editing the SVG:

```bash
rsvg-convert -w 1280 -h 640 docs/social-preview.svg -o docs/social-preview.png
```

GitHub repository social previews are uploaded manually in **repository
Settings → Social preview**; there is no API for this. Upload
`docs/social-preview.png` there.
