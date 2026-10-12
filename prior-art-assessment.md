# Prior-art assessment: scaffold workspace, project docs, branding, spec-first capability docs

Scope: the precedent in local sibling repositories that this change must copy or
adapt. Everything below was read from disk on 2026-10-11. Inputs used: `issue.md`,
`prior-art.md`, `brief.json` in this directory, and the approved plan
`/home/paudley/.claude/plans/i-would-two-create-snoopy-sifakis.md`. Nothing was
written outside this file.

## Headline findings

1. **logit-loom does not hold most of the "house templates" the plan says to copy from
   it.** `/home/paudley/Active/logit-loom` has no `.goals`, `.baseline`, `.deficiencies`,
   `CLAUDE.md`, `clippy.toml`, `deny.toml`, `REUSE.toml` or `.githooks/`. It does provide
   `Cargo.toml` `[workspace.lints]`, `rust-toolchain.toml`, `rustfmt.toml`, `AGENTS.md`,
   a Makefile and CI. The real templates for the house files are in purrdf
   (`.deficiencies` is byte-identical in purrdf and gmeow-ontology, sha256 prefix
   `091e58efd8bd`).
2. **logit-loom's CI does not test its MSRV.** `ci.yml` runs `rustup toolchain install 1.97.1`,
   but `rust-toolchain.toml` selects `nightly` and outranks `rustup default`. The jobs
   therefore run nightly. purrdf's `msrv` job documents this trap and fixes it with
   `RUSTUP_TOOLCHAIN` plus an assertion step. Copy purrdf's job, not logit-loom's.
3. **The license texts are already byte-identical across the family.** `LICENSE-MIT`
   (`958c0e61…`), `LICENSE-APACHE` (`cfc7749b…`) and `LICENSE-MULAN` (`eb7a1d71…`, the
   pinned digest) match in purrdf, gmeow-gts, visual-hashing and logit-loom (MIT and
   Apache only). `LICENSES/CC-BY-4.0.txt` exists only in purrdf (`d557539d…`).
4. **purrdf is not a REUSE-lint precedent.** It never runs `reuse lint`, and its
   `LICENSES/` has no `MIT.txt` or `Apache-2.0.txt`. The working `reuse` precedents are
   gmeow-gts and visual-hashing. Both have `REUSE.toml` and the `LICENSES/` trio.
5. **Three of the seven census copies are AGPL-3.0-only** (katamari, sparrow, and the
   gmeow-ontology branch). The plan bases raf-census on "the katamari/sparrow branch",
   and RAF is offered under `MIT OR Apache-2.0 OR MulanPSL-2.0`. Blackcat holds copyright
   in all of them, so relicensing is possible, but the owner must decide it explicitly
   (see Risks).
6. **`origin` already points at the renamed repo.** It is
   `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git`, so that
   checklist item only needs verifying.

## 1. Precedent, by area

### 1.1 Workspace manifest and lints — `/home/paudley/Active/logit-loom/Cargo.toml`

- `[workspace] resolver = "3"`. `[workspace.package]` has `edition = "2024"`,
  `rust-version = "1.97.1"`, `authors = ["Blackcat Informatics Inc."]`,
  `license = "MIT OR Apache-2.0"` (two licences, not three) and `repository`/`homepage`.
- `[workspace.lints.rust]`: `unreachable_pub = "deny"`, `missing_docs = "deny"`,
  `unsafe_code = "deny"`, `unused_qualifications = "warn"`. The explanatory comment is worth
  keeping verbatim:
  > "In a library crate every `pub` item is a dead_code lint ROOT, so `pub` silently disables
  > dead-code detection … Declared here, not as -Zcrate-attr in .cargo/config.toml: rustflags
  > apply to every unit compiled, INCLUDING path dependencies in sibling repos we do not own."
- `[workspace.lints.clippy]`: `all`/`pedantic` at `warn` with `priority = -1`, plus
  `module_name_repetitions = "allow"` and `must_use_candidate = "allow"` with **no reasons**.
  The issue requires a reason on every allow. purrdf's style puts the reason in an inline
  comment, e.g. `must_use_candidate = "allow"  # blanket #[must_use] adds noise; applied by hand where it guards bugs`
  (`/home/paudley/Active/purrdf/Cargo.toml` lines 199–230).
- purrdf adds `rust_2018_idioms = { level = "warn", priority = -1 }`, with a note that the
  underscore spelling is required. It runs `missing_docs`/`unreachable_pub` at `warn`, where
  logit-loom uses `deny`. The issue specifies the logit-loom set.
- logit-loom's `/home/paudley/Active/logit-loom/.cargo/config.toml` explains why lints must
  not go in rustflags: "a member lacking `[lints] workspace = true` silently opts out of
  EVERY workspace lint". RAF should have no `.cargo/config.toml` rustflags ("No rustflags"
  in the issue). The `[lints] workspace = true` warning still applies to every crate added
  later.

### 1.2 Toolchain

- `/home/paudley/Active/logit-loom/rust-toolchain.toml` carries the machine-policy comment:
  > "Machine policy (Patrick, 2026-09-10): Rust is ALWAYS the floating stage nightly.
  > RUSTUP_TOOLCHAIN=nightly is enforced machine-wide, so this file cannot select anything
  > else locally; it exists so CI and other checkouts follow the same floating channel."
  It sets `channel = "nightly"`, `components = ["clippy", "rustfmt"]` and `profile = "minimal"`.
- `/home/paudley/Active/purrdf/rust-toolchain.toml` explains the reasoning at length (floating,
  not dated; zero `#![feature]`; MSRV is guarded by the `msrv` CI job, "not this file").
  RAF can combine logit-loom's policy line with purrdf's "no `#![feature]`" and "MSRV job is
  the guard" paragraphs. Drop purrdf's `targets` (wasm32, aarch64).

### 1.3 rustfmt / clippy / deny

- `rustfmt.toml` (logit-loom): `edition = "2024"`, `use_field_init_shorthand = true`,
  `use_try_shorthand = true`.
- `clippy.toml` (purrdf, `/home/paudley/Active/purrdf/clippy.toml`): `msrv = "1.98"`. Its
  `disallowed-types` and `disallowed-methods` ban `DefaultHasher`/`RandomState`/`HashMap::new`
  and similar, with reasons that point at `purrdf_hash::fixed::FixedState`.
- `deny.toml`:
  - `/home/paudley/Active/nyarlathotep/deny.toml`, as named in the brief: no SPDX header,
    `MulanPSL-2.0` not in `allow`, `multiple-versions = "warn"`, and a nyarlathotep-only
    `allow-wildcard-paths = true` with a "Brainstem's private path crates" comment.
  - `/home/paudley/Active/visual-hashing/deny.toml` is the closer fit. It has an SPDX
    header, a short commented `allow` list that includes `"MulanPSL-2.0"` ("this crate's
    third disjunct"), `unused-allowed-license = "allow"`, `multiple-versions = "deny"`,
    `wildcards = "deny"`, `[advisories] yanked = "deny"` and `[sources]` restricted to
    crates.io.

### 1.4 Makefile, pre-commit hook, CI

- **Makefile.** logit-loom's `check` target is `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --all-targets --locked`, `cargo test --workspace --doc --locked`,
  and a `doc` target with `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`.
  purrdf's `hooks:` target is `git config core.hooksPath .githooks`
  (`/home/paudley/Active/purrdf/Makefile` line 88).
- **Pre-commit hook** (`/home/paudley/Active/purrdf/.githooks/pre-commit`). The reusable
  skeleton:
  - `staged=$(git diff --cached --name-only --diff-filter=ACMR)`;
  - snapshot the index with `git checkout-index --all --force --prefix="$snapshot/"`;
  - export `GIT_DIR`, `GIT_INDEX_FILE` and `GIT_WORK_TREE="$snapshot"`;
  - run `rustfmt --edition 2024 --check` on the staged `.rs` files only;
  - run the gates in parallel, logging to `$logs/$gate`.

  The header comment states the principle RAF must keep:
  > "Every gate here reads the index, not the working tree … Bypassing the hook does not
  > bypass these gates: each one below also runs in `make check` and in CI."

  Everything else in the hook is purrdf-specific: the non-Rust ratchet, the
  python-binding test, the 19 named `scripts/*.py` gates and `glossary-gate`.
- **CI** (`/home/paudley/Active/purrdf/.github/workflows/ci.yaml`):
  - The header has `permissions: contents: read` and a `concurrency` rule that never
    cancels main or tag runs.
  - `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7.0.0` runs with
    `persist-credentials: false`.
  - The toolchain step is `dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8`
    with `toolchain: "nightly" # keep in sync with rust-toolchain.toml`.
  - The `msrv` job (lines 479–518) sets `RUSTUP_TOOLCHAIN: "1.98"` on
    `cargo check --workspace --lib --locked` and adds an assertion step:
    `case "$version" in "rustc 1.98."*) ;; *) echo "::error::MSRV job resolved …"; exit 1`.
    The comment explains that `rustup default` "sits at the BOTTOM of rustup's precedence
    order — below `rust-toolchain.toml`".
  - logit-loom `ci.yml` pins `actions/checkout@3d3c42e5… # v7.0.1`. Its "Install pinned
    Rust toolchain 1.97.1" steps are defeated by the toolchain file (finding 2).

### 1.5 House files

| File | Best template | Notes |
|---|---|---|
| `AGENTS.md` | `/home/paudley/Active/logit-loom/AGENTS.md` (158 lines) | Structure: Purpose / Workspace layout / Architectural invariants / Development commands / Rust style / Testing / Documentation and release hygiene / Licensing / Change discipline. Its content is all about logit-loom (token streams, llama.cpp). |
| `CLAUDE.md` | `/home/paudley/Active/purrdf/CLAUDE.md` (34 lines) | Pattern: "Read AGENTS.md — it is the canonical contract", a quick-orientation list, and the `.deficiencies` paragraph (reusable verbatim). |
| `.goals` | `/home/paudley/Active/purrdf/.goals` (13 lines) | Reusable: GREENFIELD-FIRST; SUBSUME, EXTEND, ENHANCE; LOW/NO OPTIONALITY, HARD FAILS; the three maximums. purrdf-only: "RUST-FIRST, PYTHON-SURFACE", "RDF1.2 ONLY", "MAXIMAL PORTABILITY - Keep wasmable code". |
| `.baseline` | `/home/paudley/Active/purrdf/.baseline` (7 lines) | Holds the issue-number ban: "You MUST NEVER place github issue numbers, PR# or reference development in the ontology or any documentation in the repo." lillith_decodes' copy lacks this line. gmeow-ontology adds a "make check is expensive" paragraph. |
| `.deficiencies` | `/home/paudley/Active/purrdf/.deficiencies` (17 lines) | Byte-identical to gmeow-ontology's. Notice text plus `--- ENTRIES BELOW THIS LINE ---`, with no entries. |

None of these four dotfiles carries an SPDX header. Under `reuse lint` they need a
`REUSE.toml` annotation.

### 1.6 Licensing documents and texts

- `/home/paudley/Active/purrdf/LICENSING.md` (275 lines, `CC-BY-4.0`).
  - **Keep** (with names changed):
    - the two-truths preamble;
    - "Open-source terms" (the three-row table, the SPDX block, the CC-BY-4.0 sentence for
      documentation);
    - "About MulanPSL-2.0, precisely", including the byte-pin sentence: "byte-pinned (SHA-256
      `eb7a1d71…`, from the SPDX license-list corpus, matching the canonical text at
      `license.coscl.org.cn/MulanPSL2`) and re-verified by `scripts/check-licenses.py`";
    - the inbound "Unless you explicitly state otherwise…" clause;
    - the paragraph linking the Simplified Chinese explanation and the backtranslation record;
    - "Proprietary / commercial licensing" (licensing@blackcatinformatics.ca);
    - "Trademarks" (Apache §6, MulanPSL §3, nominative use);
    - "Contributions" (CLA);
    - "Copyright notice".
  - **Remove** (purrdf-only):
    - "Third-party material (carve-out)", meaning the vendored W3C/JSON-Schema/YAML corpora
      table and `check-corpus-frozen.py`;
    - "Notices in release artifacts", meaning `package-licenses.py`, `license-inventory.toml`,
      PyPI/npm/C bundles, the compiler stdlib report and the `unsafe-libyaml` note;
    - the GTS paragraph;
    - "Unicode data compiled into published crates";
    - the opening scope phrase "Python, WebAssembly and C bindings".
- **How the Mulan digest is verified.** In `/home/paudley/Active/purrdf/scripts/check-licenses.py`,
  lines 127–147 set `MULAN_SHA256 = "eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c"`.
  `check_mulan_text(root)` SHA-256-hashes **both** `LICENSE-MULAN` and `LICENSES/MulanPSL-2.0.txt`
  and reports `missing …` or `… digest X != pinned Y`. The docstring gives the reason:
  > "The committed MulanPSL-2.0 text is a byte contract: its Chinese text governs (its own
  > section 6), so a silently drifted copy would change the controlling terms."

  The same script also reads the expected header expression from `[workspace.package] license`
  in `Cargo.toml` (`workspace_license()`, parsed with `tomllib`) instead of restating it. The
  comment says why: a branch once merged five files at `MIT OR Apache-2.0` after the switch to
  three licences. The script runs in `make check` (lines 116–117, with `--self-test`) and in
  the pre-commit hook (gate `check-licenses`). The rest of the 820 lines is vendored-root
  logic that RAF does not need.
- `/home/paudley/Active/purrdf/docs/LICENSING.zh-Hans.md` (30 lines, CC-BY-4.0, source sha256
  `77eaedbd…`).
  - Paragraph 1 (the three choices, commercial licensing is not a fourth option, CC-BY-4.0
    for documents) and paragraph 2 (Mulan §6/§2/§3/§4 and the byte-identical text) are
    generic apart from the name `PurRDF`.
  - Paragraph 3 (Unicode-3.0, YAML MIT, XML conformance data) and paragraph 4 (Cargo, two
    Python packages, npm/WASM, C packages) are purrdf-only and must be removed or rewritten.
    That is a content change, not just a name swap.
- `/home/paudley/Active/purrdf/docs/LICENSING-BACKTRANSLATION.md` (69 lines). It records
  timestamps, source path, byte count, source SHA-256, the unchanged Mulan SHA-256, a
  "Comparison" section and the full backtranslation. Its own rule: "A change to the Chinese
  source requires updating this record and its source fingerprint". The method is "An
  independent model read the entire Chinese licensing explanation and translated it into
  English without reading the English original". No purrdf gate checks the fingerprint;
  the rule exists only as prose. gmeow-gts has the same record at
  `/home/paudley/Active/gmeow-gts/LICENSING-BACKTRANSLATION.md`.
- **Licence texts.** The purrdf `LICENSE-MIT`, `LICENSE-APACHE` and `LICENSE-MULAN` (1084,
  11358 and 9233 bytes) and `LICENSES/CC-BY-4.0.txt` can be copied byte for byte. The MIT
  text is already "Copyright (c) 2026 Blackcat Informatics® Inc.", not project-named.

### 1.7 REUSE

- `/home/paudley/Active/gmeow-gts/REUSE.toml` uses `version = 1`, `SPDX-PackageName`,
  `SPDX-PackageSupplier = "Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>"`,
  `SPDX-PackageDownloadLocation`, and `[[annotations]]` with `precedence = "aggregate"` for:
  - lockfiles, `CITATION.cff`, `.gitignore` and `.gitattributes`;
  - `docs/*.png` ("Rendered raster assets (no inline header possible)").

  Its `LICENSES/` holds `Apache-2.0.txt`, `MIT.txt` and `MulanPSL-2.0.txt`, byte-identical
  to the root `LICENSE-*` files. Note the comment: "REUSE 5 can miss them on some non-Latin
  Markdown files; keep an aggregate subtree annotation". The zh-Hans licensing doc needs
  this annotation.
- `/home/paudley/Active/visual-hashing/REUSE.toml` is the minimal form, in a Rust repo of
  RAF's size: a header, then annotations for `Cargo.lock`, `CITATION.cff` and `.gitignore`.

### 1.8 CONTRIBUTING / CODE_OF_CONDUCT / SECURITY

- `/home/paudley/Active/purrdf/CONTRIBUTING.md` (179 lines).
  - **Keep:**
    - "Ways to contribute";
    - "Versioning & releases": semver; MSRV is "a notable, changelog-recorded change that
      rides a minor bump and never ships in a patch release"; the development-toolchain
      paragraph; the conventional-commit changelog;
    - "Before you open a pull request";
    - "Licensing of contributions";
    - "Conduct".
  - **Remove:**
    - "Design constraints" (no Cargo features, kernel hygiene, determinism of the GTS
      writer, conformance corpora);
    - Node.js and `/opt` profiling notes, `make doctor`, wasm/capi targets;
    - "A SKIP is not a pass" (rewrite to fit if kept), "Benchmarks", build-dir sweeping;
    - PyPI/npm lockstep wording;
    - `docs/RELEASE.md` reference;
    - "pedantic + nursery" (RAF's set has no nursery).
- `/home/paudley/Active/purrdf/CODE_OF_CONDUCT.md` (47 lines). Name occurrences are on lines
  4, 8 and 47, plus one phrase on line 15 ("the library, the codecs, the specs"). Contact
  <conduct@blackcatinformatics.ca>.
- `/home/paudley/Active/purrdf/SECURITY.md` (151 lines).
  - **Keep verbatim:**
    - "Reporting a vulnerability" process (security@blackcatinformatics.ca, `SECURITY`
      subject line);
    - "What to expect" (48 h / 7 days);
    - "Responsible disclosure process";
    - "Reporter credit";
    - "Security updates";
    - the PGP block (key uid "Blackcat Informatics Inc. (Secure Inbound Key)").
  - **Rewrite:**
    - the surface list (RDF codecs, SPARQL, GTS, FFI, PyPI);
    - the gmeow-gts coordination sentence;
    - the "Supported versions" table. purrdf's still says `0.1.x` at version 3.0.1, which
      is stale; RAF genuinely starts at `0.1.x`.

### 1.9 GitHub templates — `/home/paudley/Active/purrdf/.github/`

- `ISSUE_TEMPLATE/bug_report.yml`: fields `what-happened`, `reproduction`, `surface`
  dropdown (Rust/Python/wasm/C), `version` and `platform`. For RAF, the surface becomes a
  crate dropdown (`raf-okf`, `raf-source`, `raf-census`, `cargo-census`, `raf-workplan`,
  `cargo-workplan`), and the CONFORMANCE.md link is removed.
- `ISSUE_TEMPLATE/feature_request.yml`: the "Two hard constraints" intro (no Cargo
  features, no minted IRIs) is purrdf-only. Replace it with RAF constraints:
  normal-form stability, closed key sets, no deferrals.
- `ISSUE_TEMPLATE/config.yml`: the security contact link. Only the URL changes.
- `PULL_REQUEST_TEMPLATE.md`: Summary and Checklist. Lines on Cargo features, `generated/`,
  `vectors/`, wasm and `purrdf_testkit::bench` are purrdf-only. Add lines for SPDX/`reuse
  lint`, capability-doc state, no issue numbers, and normal-form version bump. The comment
  "Link the issue if there is one" is fine, since it applies to the PR body, not to repo docs.

### 1.10 CITATION / cliff / CHANGELOG

- `/home/paudley/Active/purrdf/cliff.toml`.
  - Reusable verbatim:
    - the BREAKING-marker logic (`breaking_keys` resolved over the whole release before
      rendering, needed because squash merges keep branch history);
    - the dedup over `scope::message` keys;
    - the `commit_preprocessors` that strip `#NNN`, `(#NNN)` and `(closes #N)`;
    - `commit_parsers`;
    - `protect_breaking_commits = true`.
  - purrdf-only:
    - the header prose (lockstep crates.io/PyPI/npm, C ABI);
    - `tag_pattern = "rust-v[0-9]*"`;
    - the `ALCH?OIQ` → `SHOIQ(D)` rewrite;
    - the `check-issue-refs.py`/`check-doc-claims.py` references.
- `/home/paudley/Active/purrdf/CITATION.cff` is long and has a DOI. The better template is
  `/home/paudley/Active/visual-hashing/CITATION.cff`: cff 1.2.0, authors `Blackcat
  Informatics® Inc.` plus Patrick Audley (ORCID 0000-0003-4382-7625), `license:
  [Apache-2.0, MIT, MulanPSL-2.0]`, `repository-code` and keywords. Do not copy purrdf's DOI.
- `CHANGELOG.md`: purrdf's header matches the cliff header. visual-hashing's
  (`# Changelog` / "All notable changes to `visual-hashing` are recorded here." /
  `## [Unreleased]`) is the right size for a new repo.

### 1.11 Branding and logo

- `/home/paudley/Active/purrdf/docs/BRAND.md` (79 lines, CC-BY-4.0). Sections: Name,
  Positioning, Tagline, Repository description, Family system, Colour tokens, Logo assets.
  - The Family system paragraph is reusable: "reusing the shared `cat-head-core` SVG group
    verbatim and swapping only the **service object** held by the cat". It names the
    siblings `service-graph` (gmeow-ontology) and `service-chain` (gmeow-gts).
  - Tokens: cat/ink `#111214`; paper `#fffdf5`; feature `#ffffff`; accents red `#ea4335`,
    blue `#4285f4`, yellow `#fbbc05`, green `#34a853`.
  - Rebuild command: `rsvg-convert -w 1280 -h 640 docs/social-preview.svg -o docs/social-preview.png`.
    The social preview must be uploaded by hand in repository Settings (there is no API).
- `/home/paudley/Active/purrdf/docs/purrdf-logo.svg` (6708 bytes):
  - starts with two SPDX XML comments before `<svg>`; uses
    `viewBox="0 0 1024 1024" role="img" aria-labelledby="title desc"`;
  - glow filter `<filter id="soft-white-glow" x="-12%" y="-12%" width="124%" height="124%" color-interpolation-filters="sRGB">`
    with two `feDropShadow` (stdDeviation 18 / opacity 0.78, and 5 / 0.72, both white);
  - a `<style>` with classes `.cat .feature .feature-stroke .ink-stroke .node .frame .edge .content-line`
    and a colour-token comment;
  - a "Logo system note" XML comment;
  - outer `<g id="purrdf-logo" filter="url(#soft-white-glow)">`, holding in draw order:
    1. `<g id="cat-head-core">`, lines 39–107, 69 lines: `cat-silhouette`, left/right inner
       ear, eye, pupil, eye-spark, brow, `nose`, `whiskers`;
    2. `<g id="service-triple">`, holding the accent colours;
    3. `<g id="mouth-bite" class="cat">` (upper-mouth, two tiny fangs).

  **Draw order matters.** `mouth-bite` is drawn after the service group, so the cat appears
  to bite the service object's top element (the subject node at cx 512, cy 660).
  `service-toolbox` must put its handle under the bite region (roughly x 459–578, y 604–657).
- **Verbatim check.** The extracted `cat-head-core` block is 69 lines with identical sha256
  (prefix `4ec35cf6551175db`) in all three repos: `purrdf/docs/purrdf-logo.svg`,
  `gmeow-ontology/docs/gmeow-logo.svg` and `gmeow-gts/docs/gts-logo.svg`. The
  gmeow-ontology logo has no SPDX header.
- `/home/paudley/Active/purrdf/docs/social-preview.svg` (6303 bytes):
  - `viewBox="0 0 1280 640"`, gradient `bg` `#0b0f17`→`#141c2b`, filter
    `glow` (stdDeviation 14, opacity 0.55);
  - the logo is inlined at `transform="translate(48,98) scale(0.435)"` and uses
    **`class="cat-head-core"` / `class="service-triple"` instead of `id`**, so ids are not
    duplicated;
  - a text block at x≈566–570 (DejaVu Sans, which is installed locally: 22 faces), a chip
    row and a four-colour accent bar at y=628.

  The current PNG is 1280×640 RGB. gmeow-ontology's social preview uses a different, light
  design (`filter id="shadow"`, dy 18). purrdf's dark card is the one to copy.

### 1.12 OKF base definition

- `/home/paudley/Active/gmeow-gts/docs/GTS-OKF.md` defines the base format, "Knowledge
  Catalog Interop":
  > "`type:` is the only required frontmatter key; `title:`, `description:`, `resource:`,
  > `tags:`, `timestamp:`, and arbitrary producer extension keys are preserved"

  and "frontmatter-less `index.md` files are treated as navigation pages, not concepts". Its
  rejections are: symlinks, missing frontmatter (except `index.md`), non-mapping
  frontmatter, missing `type:`, unsafe relative paths, and dangling links under
  `--strict-links`.
- `/home/paudley/Active/gmeow-ontology/docs/okf.md` is the closed-profile precedent:
  > "The `okf:` contract (frontmatter keys): a **closed** vocabulary … purrdf's reader
  > validates this profile strictly — an unrecognized frontmatter key is a HARD FAIL, never a
  > silently-accepted ad-hoc predicate."
  `type` is a string literal (`Class`/`Property`/`Individual`), and the body's
  `[text](target)` links become edges.
- Example documents:
  - `/home/paudley/Active/gmeow-gts/vectors/okf/extensions/metrics.md` (`type: Metric`,
    `title`, scalar and nested extension keys);
  - `/home/paudley/Active/gmeow-gts/vectors/okf/bigquery-join/` (`datasets/`,
    `reference/`, `index.md`).
- How this maps to RAF: `type: Capability|Change`, `title` and `description` are base keys.
  `id`, `state`, `bindings`, `depends`, `changes`, `affects`, `contract_effect` and `items`
  are producer extensions, closed per type in the way gmeow does it. The generated
  `docs/capabilities/index.md` should stay **frontmatter-less** so it stays a valid OKF
  navigation page. `docs/architecture/okf-profile.md` should say whether `id: cap:<slug>`
  stands in for OKF's `resource` (an IRI) or exists alongside it.

### 1.13 GROUNDING coverage duty — `/home/paudley/Active/gmeow-ontology/docs/GROUNDING.md` §"The coverage duty" (lines 232–252)

> "Design-promise direction: every surface a design doc promises is realized, explicitly gated
> as design-only in the doc itself, or carried by a wired issue. Silent omission from a slice's
> own status table is a defect."
> "Built-beyond-docs direction: … when it does, the documents follow the built form. A stale
> "queued" or "deferred" marker for work that has landed is a defect of the same weight as an
> unrealized promise."

The duty is gated, not left to vigilance: "A design-set-table entry with **no** realized-state
marker misses the dimension, so the silent omission … is a scored, gating defect". An example
realized-state column is in
`/home/paudley/Active/gmeow-ontology/slices/grounding/lang/docs.md` lines 45–55
(`| Document | Genre | Realized state | Contents |`).

How this maps to RAF:

| GROUNDING | RAF |
|---|---|
| realized-state marker | required `state` key (`considered / designed / implementing / delivered`); a missing key is a hard fail |
| "carried by a wired issue" | carried by a `work/changes/<slug>.md` record, because issue numbers are banned |
| stale-marker defect | `delivered` with leftover PROPOSED blocks |
| built-beyond-docs direction | the M2 capability `workplan-two-way-coverage` |

### 1.14 Helper-census documentation (input to the census capability docs)

- `/home/paudley/Active/purrassist/docs/HELPER-CENSUS.md` (238 lines) is the most complete
  prose spec. Sections: Commands and enforcement; Detection and traversal (syn walk,
  `#[path]`, structural and shim forms, ">= 30 normalized tokens", shims ">= six tokens",
  integer literals compared by value, hex tables); Ledger schema (the `[[job]]` required
  keys, `[[job.variant]]`, `[[distinct]]`, and the criteria `a`/`b`/`c`/`layering`);
  Provenance (`UPSTREAM.json` SHA-256 pins). This is the template for `census-ledger`,
  `census-detectors` and `census-normal-form`.
- `/home/paudley/Active/sparrow/docs/helper-census.md` (95 lines) is the reference-scan
  precedent: `--cross` runs `git archive` on a pinned purrdf commit into
  `target/helper-census/purrdf-<rev>/`. Reports are marked **UNPINNED** or **NOT CHECKED**.
  This is the template for `census-reference-scan`, which the plan replaces with
  `cargo metadata`.
- `/home/paudley/Active/purrdf/helpers-ledger.toml` lines 1–60 document every job key in a
  header comment: `id`, `summary`, `home`, `entry_points`, `spec`, `vectors`, `bench`,
  `sites`, `replaces_external`, `enforced`, and `forbidden.{constants,fingerprints,names}`
  with the `rule:*` catalogue. These are the purrdf-dialect keys the plan moves into
  `[job.ext]`.
- The debt-ratchet precedent is
  `/home/paudley/Active/gmeow-ontology/.worktrees/helper-census/helpers-census-debt.json`
  (`{"groups":[{"kind":"body","members":[…]}]}`), on the unmerged `agent/helper-census` branch.

## 2. Reuse verbatim

Change only the SPDX license expression where noted, and the project name.

- `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-MULAN` from purrdf (or gmeow-gts or
  visual-hashing; all are byte-identical). `LICENSE-MULAN` must hash to `eb7a1d71…`.
- `LICENSES/MIT.txt`, `LICENSES/Apache-2.0.txt`, `LICENSES/MulanPSL-2.0.txt` from
  `/home/paudley/Active/gmeow-gts/LICENSES/`.
- `LICENSES/CC-BY-4.0.txt` from `/home/paudley/Active/purrdf/LICENSES/CC-BY-4.0.txt`
  (sha256 `d557539d…`).
- The `cat-head-core` group: lines 39–107 of `/home/paudley/Active/purrdf/docs/purrdf-logo.svg`.
  Also verbatim: the `mouth-bite` group, the `soft-white-glow` filter, the `<style>` classes
  `.cat .feature .feature-stroke` and the colour-token comment.
- The social-preview frame: background gradient, `glow` filter, class-based inlining of the
  logo, and the accent bar.
- `.deficiencies` from purrdf (identical to gmeow-ontology's).
- The `.baseline` lines from purrdf: adversarial gap analysis, no deferrals, the
  issue-number ban, "when you see it, you own it".
- The `.deficiencies` paragraph and the "Read AGENTS.md — canonical contract" line from
  `/home/paudley/Active/purrdf/CLAUDE.md`.
- The logit-loom `[workspace.lints.rust]` block with its `unreachable_pub` comment, and the
  `rustfmt.toml` body.
- The logit-loom `rust-toolchain.toml` machine-policy comment and `[toolchain]` table.
- purrdf CI's `msrv` job shape (`RUSTUP_TOOLCHAIN` env plus the assertion step) and the
  CI header (permissions, concurrency, pinned action SHAs, `persist-credentials: false`).
  Only the version string changes, to `1.97.1`.
- The purrdf pre-commit snapshot skeleton: the staged list, `checkout-index` snapshot, GIT_*
  export, staged-only rustfmt, parallel gate loop, header principle text. Also the
  `hooks: git config core.hooksPath .githooks` target.
- The `SECURITY.md` process sections and PGP block. `CODE_OF_CONDUCT.md` apart from the
  name (and the one scope phrase on line 15).
- The `cliff.toml` template body, `commit_preprocessors` (minus the `ALCH?OIQ` rule),
  `commit_parsers` and `protect_breaking_commits`.
- `LICENSING.md` sections: Proprietary / commercial, Trademarks, Contributions, Copyright
  notice, and "About MulanPSL-2.0, precisely" (except the script name, see Adapt).
- The OKF base rules (`type` required; frontmatter-less `index.md` is navigation) and
  gmeow's "unrecognized key is a HARD FAIL" doctrine.

## 3. Adapt (what to change)

| Target | Source | Change |
|---|---|---|
| `Cargo.toml` | logit-loom | `license = "MIT OR Apache-2.0 OR MulanPSL-2.0"`; repository URL to Blackcat-Informatics/rust-agentic-foundations; add a reason comment on every clippy allow (purrdf inline style); no `[workspace.dependencies]` beyond what M0 needs; no `[profile.*]` llama tuning. |
| `rust-toolchain.toml` | logit-loom + purrdf | Add purrdf's "zero `#![feature]`" and "MSRV job is the guard" sentences, and SPDX with three licences. No `targets`. |
| `clippy.toml` | purrdf | `msrv = "1.97.1"`, not 1.98. Rewrite the disallowed-type/method reasons to name RAF's remedy (`BTreeMap`/`BTreeSet`, or an explicit fixed hasher), not `purrdf_hash`. |
| `deny.toml` | visual-hashing (preferred) / nyarlathotep | Add SPDX; keep `MulanPSL-2.0` in `allow`; drop nyarlathotep's brainstem `allow-wildcard-paths` comment; keep `unused-allowed-license = "allow"` (the graph is near empty in M0). |
| `Makefile` | logit-loom + purrdf | `check`: fmt, clippy `-D warnings`, test (`--all-targets` + `--doc`), doc, `cargo deny check`, `reuse lint`, Mulan digest check, issue-reference grep. `hooks` target. Drop `models-check`/`package-*`. |
| `.githooks/pre-commit` | purrdf | Remove the helper-census ratchet, python-binding test and all 19 purrdf scripts. Gates: staged rustfmt, `reuse lint` on the snapshot, Mulan digest, issue-reference scan. Later issues add the self-census/self-workplan gates. |
| `.github/workflows/ci.yaml` | purrdf | Jobs: fmt, clippy, test, doc, deny, msrv (1.97.1), plus reuse lint if not folded into a job. Drop `fetch-depth: 0` (the ratchet is gone) unless the workplan reconcile job needs history later. |
| `AGENTS.md` | logit-loom (structure) | Rewrite Purpose, layout (`crates/raf-*`, `docs/capabilities`, `docs/architecture`, `work/changes`), invariants (normal-form versioning, closed key sets, dogfood gates, completion gate), commands, licensing (three licences + CC-BY docs). Drop the llama.cpp, model and efficacy content. |
| `CLAUDE.md` | purrdf | Keep the pointer line and the `.deficiencies` paragraph; rewrite the quick orientation for RAF. Drop binary-size, PyO3, wasm and RDF items. |
| `.goals` | purrdf | Drop the RDF1.2 ONLY, wasm portability and Python-surface lines; add RAF-relevant goals (dev-dependency usability, determinism of fingerprints). |
| `LICENSING.md` | purrdf | PurRDF→RAF, purrdf→raf. Remove the carve-out, release-artifact, GTS and Unicode sections and the bindings scope. Replace "re-verified by `scripts/check-licenses.py`" with RAF's actual check (Makefile/hook/test). |
| `docs/LICENSING.zh-Hans.md` | purrdf | Substitute the name, and **remove paragraphs 3–4** (Unicode/YAML/XML; Cargo/Python/npm/C packages). This is a real Chinese text change, so it needs a fresh backtranslation. |
| `docs/LICENSING-BACKTRANSLATION.md` | purrdf/gmeow-gts | New record: new timestamps, byte count and source SHA-256 of RAF's zh file; Mulan SHA unchanged; produced by an independent model that sees only the Chinese text, never by editing purrdf's English. |
| `CONTRIBUTING.md` | purrdf | New Design constraints (normal-form stability with a version bump, dogfood gates, workplan completion gate, closed OKF keys). MSRV 1.97.1. Drop Node, doctor, bench, build-dir, PyPI/npm, RELEASE.md and "nursery". Choose the SPDX (see Risks). |
| `SECURITY.md` | purrdf | Rewrite the surface: untrusted Rust source (syn), TOML/YAML frontmatter, `#[path]` and reference-scan path traversal, spawned `cargo`/`git` subprocesses, the crates.io supply chain. Supported versions `0.1.x`. Drop the gmeow-gts coordination line. |
| `docs/BRAND.md` | purrdf | Name **RAF (Rust Agentic Foundations)**; there is no pun-casing rule like PurRDF's. Machine ids `raf-*`/`cargo-*`. `service-toolbox` replaces `service-triple`, with accents isolated to it. Asset names `docs/raf-logo.svg`, `docs/social-preview.{svg,png}`. |
| `docs/raf-logo.svg` | purrdf logo | New `<title>`/`<desc>`, outer `<g id="raf-logo">`, a new `<g id="service-toolbox">` (toolbox + wrench, four accents) drawn between `cat-head-core` and `mouth-bite`, and an updated logo-system note. |
| `docs/social-preview.svg` | purrdf | Text block (RAF / tagline / crate list), chips as crate names instead of languages, licence line. Inline the logo with `class=` attributes. |
| GitHub templates | purrdf | The crate dropdown replaces the language-surface dropdown; RAF constraints in the feature intro; PR checklist items for RAF; URLs. |
| `CITATION.cff` | visual-hashing | RAF title, abstract and keywords; `version: "0.1.0"`; no DOI. |
| `cliff.toml` | purrdf | Header prose for RAF; `tag_pattern` for RAF's tag scheme (e.g. `v[0-9]*`, or per-crate if crates version independently); drop `ALCH?OIQ`. |
| `REUSE.toml` | gmeow-gts / visual-hashing | Annotate `Cargo.lock`, `CITATION.cff`, `.gitignore`, `.goals`, `.baseline`, `.deficiencies`, `docs/*.png`, and the zh-Hans doc (aggregate annotation, the REUSE 5 non-Latin issue). |
| Capability docs | purrassist/sparrow census docs + `INITIAL_PLANNING_PLAN.md` | Write in present tense as contracts; do not copy provenance, historical counts or "qualification report" narrative. Fix the threshold story: v1 uses `min_tokens` 20, not purrassist/decodes' 30. |

## 4. Recurrence

- **Defect-class recurrence: UNKNOWN.** `brief.json` reports no `GhpRsq-Defect-Class`
  trailers in this repository's history. This is the first recorded instance, not
  "attempt 1" and not zero.
- **Recurrence of the duplication problem this repo exists to solve: counted, 7 copies** of
  purrdf's helper-census tool, about 54.5k lines of Rust in total.

| Copy | Path | .rs lines | Fingerprint hash | `MIN_TOKENS` | File SPDX |
|---|---|---|---|---|---|
| purrdf (origin) | `/home/paudley/Active/purrdf/crates/helper-census` | 13,065 | BLAKE3 | 20 | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| gmeow-ontology (unmerged branch `agent/helper-census`) | `/home/paudley/Active/gmeow-ontology/.worktrees/helper-census/crates/helper-census` | 7,724 | BLAKE3 via `purrdf_hash::blake` | 20 | AGPL-3.0-only |
| lillith_memetics | `/home/paudley/Active/lillith_memetics/crates/helper-census` | 8,005 | BLAKE3 | 20 | MIT OR Apache-2.0 OR MulanPSL-2.0 (workspace `LicenseRef-Proprietary`) |
| purrassist | `/home/paudley/Active/purrassist/crates/helper-census` | 6,425 | **SHA-256** (`purrassist_core::digest::Digest`) | **30** | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| lillith_decodes | `/home/paudley/Active/lillith_decodes/crates/helper-census` | 3,316 | BLAKE3 | **30** | MIT OR Apache-2.0 OR MulanPSL-2.0 |
| katamari | `/home/paudley/Active/katamari/crates/katamari-helper-census` | 7,895 | BLAKE3 | 20 | **AGPL-3.0-only** |
| sparrow (ported from katamari) | `/home/paudley/Active/sparrow/crates/sparrow-helper-census` | 8,056 | BLAKE3 | 20 | **AGPL-3.0-only** |

`/home/paudley/Active/purrdf-408-wip-source/crates/helper-census` is a working snapshot of
purrdf and is not counted.

## 5. Prior attempts and why they did not hold

**Searched.** I looked for `~/Active` directory names matching `*tool*`, `*found*`,
`*workplan*`, `*shared*`, `*common*`, `*template*` and `*scaffold*`, and grepped Markdown for
`workplan`, "capability document" and "PROPOSED change". The only workplan hits were
unrelated (`.stage` comments, fenris claudedocs). **No earlier Rust shared dev-tooling
workspace and no workplan tool exist locally.**

The earlier attempts at sharing were these:

1. **Vendoring with provenance pins.**
   - purrassist pins file SHA-256s to purrdf commit `0387e83e…` in
     `crates/helper-census/UPSTREAM.json`.
   - katamari's `main.rs` says "Ported from purrdf's `crates/helper-census` at purrdf revision …".
   - sparrow: "ported from Katamari commit `f5290a43…`, with no local changes in the copied
     crate".

   Pins record where a copy came from. They do not carry later fixes forward, so each copy
   drifted at once:
   - **Hash drift.** purrassist's zero-third-party-dependency policy forced SHA-256 from its
     own core crate. Its doc admits: "They differ from the source tool's BLAKE3 ids; upstream
     ids must not be pasted into this ledger." Fingerprints are therefore not comparable
     across repos.
   - **Threshold drift.** `MIN_TOKENS` is 20 in purrdf, katamari, sparrow, memetics and
     gmeow, but 30 in purrassist and decodes. The same code groups differently.
   - **Ledger dialects.**
     - purrdf requires `spec/vectors/bench/sites/replaces_external`.
     - purrassist requires the same keys but forces `sites`/`replaces_external` to be empty.
     - lillith_decodes renamed the file to `tools/duplicate-homes.toml` and added
       `disposition = folded|owned|sanctioned` plus `owner`.
     - memetics added `--reference PREFIX`.
     - katamari/sparrow added `--cross` with a `git archive` pin.
     - gmeow added a debt-ratchet JSON.
   - **Licence drift.** Three copies were relabelled AGPL-3.0-only.
   - **Duplicated parsers.** purrassist keeps a "strict first-party TOML subset" reader,
     checked against Python `tomllib`.
   - **Improvements stay local.** purrdf's newer `rules.rs`, the `Tree` trait and
     `--self-test` never reached the forks.
2. **Reference scanning instead of copying.** katamari/sparrow `--cross` and memetics
   `--reference` let one repo see another's helpers. Each repo still carries its own copy of
   the tool, so this solved cross-repo *detection* but not the duplicated *tool*. The plan
   folds both into one `cargo metadata`-pinned reference model.
3. **The tool as a library coupled to purrdf.** The gmeow-ontology branch depends on
   `purrdf_hash::{blake,hex,frame}`. That ties a dev tool to a runtime library's release
   cadence. It is still unmerged, two commits ahead of `main`.
4. **House precedent for publish-and-depend.** `/home/paudley/Active/pyqa_lint` is Blackcat's
   Python QA harness. It is published to PyPI and can also be vendored as a submodule. It is
   the in-house model for RAF's approach: publish versioned crates and consume them as
   dev-dependencies, so every repo gets fixes through a version bump instead of a re-port.

**Why RAF's approach should hold where vendoring did not.** There is one published source, a
versioned normal form (`normal_form = 1`, with a hard fail on mismatch plus `migrate`), a
strict core schema with registered `[job.ext]` extensions in place of forked dialects, and
pinned fingerprint goldens. The dialects become named extensions instead of forks.

## 6. Tool availability (local, reported only)

| Tool | Status |
|---|---|
| `reuse` | **MISSING** (not on PATH; `python3 -c 'import reuse'` fails). `uvx` 0.11.28 is present, so `uvx reuse lint` is possible with network access. |
| `cargo-deny` | present, 0.20.2 |
| `cargo-audit` | present, 0.22.2 |
| `rsvg-convert` | present, 2.63.2 (cairo 1.18.6, pango 1.58.2); DejaVu fonts installed |
| `markdownlint-cli2` | MISSING (`npx` 11.20.0 present) |
| `codespell` | MISSING |
| `git-cliff` | present, 2.13.1 |
| `cargo-msrv`, `typos`, `lychee` | MISSING |
| `gh` | present, 2.102.0 |
| `rust-validate` | present, `/home/paudley/stage/root/bin/rust-validate`: the only sanctioned way to run an old toolchain locally (`rust-validate 1.97.1 cargo check …`), using a separate rustup home |
| `rustup toolchain list` | `nightly-x86_64-unknown-linux-gnu (active)`, `nightly-2026-09-14-x86_64-unknown-linux-gnu (default)`; rustc 1.100.0-nightly (4b6d04e70 2026-09-13), cargo 1.100.0-nightly |

A machine hook blocks any shell command that sets a non-nightly toolchain override. The
local MSRV check must therefore go through `rust-validate`. CI uses the `RUSTUP_TOOLCHAIN`
env in YAML.

## 7. Risks

1. **"`make check` passes on the empty workspace" conflicts with "no crate code" (out of
   scope).** A virtual manifest with no members should make `cargo fmt`, `clippy`, `test`
   and `doc` fail with a "virtual manifest … has no members" style error (verify). `--locked`
   needs a `Cargo.lock`, and `cargo deny check` needs a resolvable graph. The planner must
   choose: a minimal publish=false member (which is crate code), a Makefile that refuses
   gracefully (which would be a conditional skip, against "HARD FAILS"), or creating the
   first `raf-okf` skeleton with only docs. Settle this before implementation.
2. **`reuse` is not installed** and `.goals` forbids optional gates. `make check` and the hook
   will hard-fail locally until `reuse` is provided, e.g. `uvx reuse@<pinned> lint` or a
   pipx install documented in CONTRIBUTING. In CI, use a SHA-pinned `fsfe/reuse-action`.
   `reuse lint` needs every licence identifier present in `LICENSES/`: MIT, Apache-2.0,
   MulanPSL-2.0 and CC-BY-4.0. purrdf's `LICENSES/` lacks MIT and Apache, so it cannot be
   the model.
3. **MSRV.** Copying logit-loom's CI tests nothing (finding 2). Use purrdf's job with
   `1.97.1`. `clippy.toml` must say `1.97.1`, not purrdf's `1.98`. A local `make msrv`
   cannot use `RUSTUP_TOOLCHAIN` (hook-blocked) and has to call `rust-validate 1.97.1`, which
   does not exist in CI. Keep the two paths separate.
4. **The documentation licence split is inconsistent in purrdf.**
   - CC-BY-4.0: `LICENSING.md`, `BRAND.md`, zh, backtranslation.
   - `MIT OR Apache-2.0 OR MulanPSL-2.0`: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
     `SECURITY.md`, the PR template and both logo SVGs.
   - gmeow-gts annotates `docs/*.png` as `MIT OR Apache-2.0`.

   The plan says documents are CC-BY-4.0. Pick one rule (e.g. all `*.md` under CC-BY-4.0;
   SVG/PNG under the code licence or CC-BY-4.0, stated once), record it in `LICENSING.md`
   and `REUSE.toml`, and apply it consistently. Otherwise the inconsistency gets copied.
5. **The Mulan pin needs a live check.** purrdf enforces it in a Python script RAF will not
   copy. Without a replacement (e.g. `sha256sum -c` in `make check` and the hook, over both
   `LICENSE-MULAN` and `LICENSES/MulanPSL-2.0.txt`), the `LICENSING.md` sentence "re-verified
   by …" becomes a false claim.
6. **The backtranslation must be independent and freshly fingerprinted.** Removing
   paragraphs 3–4 of the zh file changes the source, so a new backtranslation must be
   produced by a model that sees only the Chinese text. purrdf has no gate on the recorded
   source SHA. Consider having `make check` compare the recorded SHA against the file.
7. **AGPL provenance.** katamari, sparrow and the gmeow branch census files are
   `AGPL-3.0-only`; RAF is tri-licensed permissive. The copyright holder is Blackcat in all
   cases, so relicensing is the owner's call, but it must be an explicit, recorded decision.
   Alternatively, derive from the tri-licensed purrdf, memetics, purrassist and decodes
   copies. The capability docs' Explanation sections should state provenance without
   implying AGPL-derived code under a permissive grant. The code concern belongs to the
   later census issues; the doc wording belongs here.
8. **Process-reference ban.** purrdf's `/home/paudley/Active/purrdf/scripts/check-issue-refs.py`
   bans `#NNN`, tracker URLs, `Task N`, `EPIC`, "this branch", "the plan's", "the plan §" and
   `AC1`. Capability docs migrated from `INITIAL_PLANNING_PLAN.md` and the issue checklist
   are likely to pick up exactly these phrases. RAF has no such gate yet, so add at least a
   grep to `make check` for the issue-number rule the acceptance criteria require.
9. **Nothing mechanically checks the "7 sections" acceptance criterion** until
   `raf-workplan` exists. Without an interim check it rests on review alone. If a check is
   added, it must not become a second schema that later disagrees with `raf-okf`.
10. **Logo layering.** `service-toolbox` must sit between `cat-head-core` and `mouth-bite`,
    with its handle under the bite region. Otherwise the bite reads wrong. In the social
    preview, use `class=` (not `id=`) for the inlined groups, as purrdf does, to avoid
    duplicate ids. The two SPDX XML comments before `<svg>` are accepted by `rsvg-convert`
    (purrdf renders them today).
11. **Changelog with squash merges.** Squash subjects carry `(#N)`. Keep cliff's
    preprocessors and the whole-release BREAKING resolution, and change `tag_pattern` away
    from `rust-v*`.
12. **Template choice for `deny.toml`.** nyarlathotep's file, as named in the brief, has no
    SPDX header and no MulanPSL entry, and carries an unrelated brainstem exception.
    visual-hashing's is the better base.
13. **Action pins.** purrdf pins checkout v7.0.0 (`9c091bb…`) and logit-loom v7.0.1
    (`3d3c42e…`). Choose one SHA-pinned set.
